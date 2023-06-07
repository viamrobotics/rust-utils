mod parse;
mod strings;

use anyhow::{anyhow, Result};
use clap::Parser;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use serde_json::{from_str, Value};
use std::{fs, io, path::PathBuf};
use tokio::time::Instant;
use viam::rpc::dial::{self, ViamChannel};
use webrtc::stats;

/// dialdbg gives information on how to connect to a Viam robot
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Filepath for remote configuration JSON containing URI and secret
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Whether direct gRPC connection should not be examined
    #[arg(long, action, conflicts_with("nowebrtc"))]
    nogrpc: bool,

    /// Whether WebRTC connection should not be examined
    #[arg(long, action, conflicts_with("nogrpc"))]
    nowebrtc: bool,

    /// Filepath for output of dialdbg (file will be overwritten)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Secret with which to connect to the robot
    #[arg(short, long, required_unless_present("config"))]
    secret: Option<String>,

    /// URI of the robot to connect to
    #[arg(short, long, required_unless_present("config"))]
    uri: Option<String>,
}

fn read_config(config: PathBuf) -> Result<(String, String)> {
    let (uri, secret): (String, String);
    let r: Value = from_str(fs::read_to_string(config)?.as_str())?;
    if let Some(u) = r.get("address") {
        // We use some funky as_str -> to_string logic as serde_json weirdly wraps to_string'ed
        // Values in quotes. Also, we want to verify "address" and "secret" are actually strings.
        // Was I too lazy to use regular serde? Yes.
        if let Some(uri_str) = u.as_str() {
            uri = uri_str.to_string();
        } else {
            return Err(anyhow!(
                "expected 'address' value to be a string in provided config"
            ));
        }
    } else {
        return Err(anyhow!("no top-level 'address' value in provided config"));
    }
    if let Some(s) = r.get("secret") {
        if let Some(sec_str) = s.as_str() {
            secret = sec_str.to_string();
        } else {
            return Err(anyhow!(
                "expected 'secret' value to be a string in provided config"
            ));
        }
    } else {
        return Err(anyhow!("no top-level 'secret' value in provided config"));
    }
    Ok((uri, secret))
}

async fn dial_grpc(uri: &str, secret: &str) {
    let creds = dial::RPCCredentials::new(
        None,
        "robot-location-secret".to_string(),
        secret.to_string(),
    );

    // `connect` may propagate an error here; log the error with a prefix so we can still
    // process logs and not immediately return from the main function.
    if let Err(e) = dial::DialOptions::builder()
        .uri(uri)
        .with_credentials(creds)
        .disable_webrtc()
        .allow_downgrade()
        .connect()
        .await
    {
        log::error!("{}: {e}", strings::DIAL_ERROR_PREFIX);
    }
}

async fn dial_webrtc(uri: &str, secret: &str) -> Option<stats::StatsReport> {
    let creds = dial::RPCCredentials::new(
        None,
        "robot-location-secret".to_string(),
        secret.to_string(),
    );

    // `connect` may propagate an error here; log the error with a prefix so we can still
    // process logs and not immediately return from the main function. Assuming there was
    // no error, return the stats report of the underlying RTCPeerConnection.
    match dial::DialOptions::builder()
        .uri(uri)
        .with_credentials(creds)
        .allow_downgrade()
        .connect()
        .await
    {
        Ok(c) => match c {
            ViamChannel::WebRTC(c) => Some(c.get_stats().await),
            _ => None,
        },
        Err(e) => {
            log::error!("{}: {e}", strings::DIAL_ERROR_PREFIX);
            None
        }
    }
}

fn output_connection_stats(stats: stats::StatsReport, out: &mut Box<dyn io::Write>) -> Result<()> {
    // StatsReport contains 13 types of stat reports; there may be more relevant stats to print
    // here, but for now I have stuck with only printing the candidates.
    writeln!(out, "\nnominated ICE candidates:\n")?;
    let now = Instant::now();
    for value in stats.reports.into_values() {
        match value {
            stats::StatsReportType::LocalCandidate(lcs) => {
                writeln!(out, "\tlocal ICE candidate:")?;
                writeln!(out, "\t\tIP address: {}", lcs.ip)?;
                writeln!(out, "\t\tport: {}", lcs.port)?;
                writeln!(
                    out,
                    "\t\tnominated {:#?} ago",
                    now.duration_since(lcs.timestamp)
                )?;
                writeln!(out, "\t\trelay protocol: {}", lcs.relay_protocol)?;
                writeln!(out, "\t\tnetwork type: {}", lcs.network_type)?;
            }
            stats::StatsReportType::RemoteCandidate(rcs) => {
                writeln!(out, "\tremote ICE candidate:")?;
                writeln!(out, "\t\tIP address: {}", rcs.ip)?;
                writeln!(out, "\t\tport: {}", rcs.port)?;
                writeln!(
                    out,
                    "\t\tnominated {:#?} ago",
                    now.duration_since(rcs.timestamp)
                )?;
                writeln!(out, "\t\trelay protocol: {}", rcs.relay_protocol)?;
                writeln!(out, "\t\tnetwork type: {}", rcs.network_type)?;
            }
            _ => {}
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let mut uri = args.uri.unwrap_or_default();
    let mut secret = args.secret.unwrap_or_default();
    if args.config.is_some() {
        (uri, secret) = read_config(args.config.unwrap())?;
    }

    // Write to output file or STDOUT if none is provided.
    let mut out: Box<dyn io::Write> = match args.output {
        Some(output) => match fs::File::create(output) {
            Ok(output_file_writer) => Box::new(output_file_writer),
            Err(e) => {
                return Err(anyhow!("error opening --output file: {e}"));
            }
        },
        None => Box::new(io::stdout()),
    };

    let mut log_config_setter = None;
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

        dial_grpc(uri.as_str(), secret.as_str()).await;
        parse::parse_grpc_logs(log_path.clone(), &mut out)?;

        // Remove temp log file after parsing.
        fs::remove_file(log_path)?;

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
        if log_config_setter.is_some() {
            log_config_setter.unwrap().set_config(config);
        } else {
            log4rs::init_config(config)?;
        }

        let sr = dial_webrtc(uri.as_str(), secret.as_str()).await;
        parse::parse_webrtc_logs(log_path.clone(), &mut out)?;

        if let Some(sr) = sr {
            output_connection_stats(sr, &mut out)?;
        }

        // Remove temp log file after parsing.
        fs::remove_file(log_path)?;

        writeln!(out, "\nDone debugging dial with WebRTC.")?;
    }

    Ok(())
}
