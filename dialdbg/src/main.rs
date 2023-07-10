mod parse;
mod stats;
mod strings;

use anyhow::{bail, Result};
use clap::Parser;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use std::{fs, io, path::PathBuf};
use viam::rpc::dial::{self, ViamChannel};

/// dialdbg gives information on how rust-utils' dial function makes connections.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Whether direct gRPC connection should not be examined. If not provided, gRPC connection
    /// will be examined.
    #[arg(long, action, conflicts_with("nowebrtc"), display_order(1))]
    nogrpc: bool,

    /// Whether WebRTC connection should not be examined. If not provided, WebRTC connection will
    /// be examined.
    #[arg(long, action, conflicts_with("nogrpc"))]
    nowebrtc: bool,

    /// Filepath for output of dialdbg (file will be overwritten). If not provided, dialdbg will
    /// output to STDOUT.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Credential payload with which to connect to the URI. If not provided, dialdbg will dial without
    /// credentials.
    #[arg(short, long)]
    credential: Option<String>,

    /// Type of credential with which to connect to the URI. Can only be provided with
    /// "--credential". If "--credential" is provided but "--credential-type" is not,
    /// credential type will default to "robot-location-secret".
    #[arg(short('t'), long, requires("credential"))]
    credential_type: Option<String>,

    /// URI to dial. Must be provided.
    #[arg(short, long, required(true), display_order(0))]
    uri: Option<String>,
}

async fn dial_grpc(uri: &str, credential: &str, credential_type: &str) {
    let dial_result = match credential {
        "" => {
            dial::DialOptions::builder()
                .uri(uri)
                .without_credentials()
                .disable_webrtc()
                .allow_downgrade()
                .connect()
                .await
        }
        _ => {
            let creds = dial::RPCCredentials::new(
                None,
                credential_type.to_string(),
                credential.to_string(),
            );
            dial::DialOptions::builder()
                .uri(uri)
                .with_credentials(creds)
                .disable_webrtc()
                .allow_downgrade()
                .connect()
                .await
        }
    };

    // `connect` may propagate an error here; log the error with a prefix so we can still
    // process logs and not immediately return from the main function.
    if let Err(e) = dial_result {
        log::error!("{}: {e}", strings::DIAL_ERROR_PREFIX);
    }
}

async fn dial_webrtc(
    uri: &str,
    credential: &str,
    credential_type: &str,
) -> Option<stats::StatsReport> {
    let dial_result = match credential {
        "" => {
            dial::DialOptions::builder()
                .uri(uri)
                .without_credentials()
                .allow_downgrade()
                .connect()
                .await
        }
        _ => {
            let creds = dial::RPCCredentials::new(
                None,
                credential_type.to_string(),
                credential.to_string(),
            );
            dial::DialOptions::builder()
                .uri(uri)
                .with_credentials(creds)
                .allow_downgrade()
                .connect()
                .await
        }
    };

    // `connect` may propagate an error here; log the error with a prefix so we can still
    // process logs and not immediately return from the main function. Assuming there was
    // no error, return the stats report of the underlying RTCPeerConnection.
    match dial_result {
        Ok(c) => match c {
            ViamChannel::WebRTC(c) => Some(stats::StatsReport(c.get_stats().await)),
            _ => None,
        },
        Err(e) => {
            log::error!("{}: {e}", strings::DIAL_ERROR_PREFIX);
            None
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let uri = args.uri.unwrap_or_default();
    let credential = args.credential.unwrap_or_default();
    let credential_type = args
        .credential_type
        .unwrap_or("robot-location-secret".to_string());

    // Write to output file or STDOUT if none is provided.
    let mut out: Box<dyn io::Write> = match args.output {
        Some(output) => match fs::File::create(output) {
            Ok(output_file_writer) => Box::new(output_file_writer),
            Err(e) => {
                bail!("error opening --output file: {e}");
            }
        },
        None => Box::new(io::stdout()),
    };

    let mut log_config_setter: Option<log4rs::Handle> = None;
    if !args.nogrpc {
        writeln!(out, "\nDebugging dial with basic gRPC...\n")?;
        // Start logger with Debug-level logging and append logs to a file in a temp directory.
        let log_path = std::env::temp_dir().join("grpc_temp.log");
        let logfile = FileAppender::builder().build(log_path.clone())?;
        let config = Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(
                Root::builder()
                    .appender("logfile")
                    .build(log::LevelFilter::Debug),
            )?;
        log_config_setter = Some(log4rs::init_config(config)?);

        dial_grpc(uri.as_str(), credential.as_str(), credential_type.as_str()).await;
        let grpc_res = parse::parse_grpc_logs(log_path.clone(), &mut out)?;
        write!(out, "{grpc_res}")?;

        // Remove temp log file after parsing if it exists.
        if let Ok(_) = log_path.try_exists() {
            fs::remove_file(log_path)?;
        }

        writeln!(out, "\nDone debugging dial with basic gRPC.")?;
    }
    if !args.nowebrtc {
        writeln!(out, "\nDebugging dial with WebRTC...\n")?;
        // Start logger with Debug-level logging and append logs to a file in a temp directory.
        let log_path = std::env::temp_dir().join("webrtc_temp.log");
        let logfile = FileAppender::builder().build(log_path.clone())?;
        let config = Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(
                Root::builder()
                    .appender("logfile")
                    .build(log::LevelFilter::Debug),
            )?;

        // Logging may have been initialized by gRPC, in which case we should use the
        // log4rs::Handle to set a new config.
        if let Some(log_config_setter) = log_config_setter {
            log_config_setter.set_config(config);
        } else {
            log4rs::init_config(config)?;
        }

        let sr = dial_webrtc(uri.as_str(), credential.as_str(), credential_type.as_str()).await;
        let wrtc_res = parse::parse_webrtc_logs(log_path.clone(), &mut out)?;
        write!(out, "{wrtc_res}")?;
        if let Some(sr) = sr {
            write!(out, "{sr}")?;
        }

        // Remove temp log file after parsing if it exists.
        if let Ok(_) = log_path.try_exists() {
            fs::remove_file(log_path)?;
        }

        writeln!(out, "\nDone debugging dial with WebRTC.")?;
    }

    Ok(())
}
