use anyhow::{bail, Result};
use chrono::{DateTime, Duration, FixedOffset};
use std::{fmt, fs, io, net::SocketAddr, path::PathBuf};
use viam::rpc::log_prefixes;

const DEVELOPMENT: Option<&'static str> = option_env!("DIALDBG_DEVELOPMENT");

// This prefix is prepended in dialdbg when connect returns an error. It is not
// from dial itself.
pub(crate) const DIAL_ERROR_PREFIX: &'static str = "unexpected dial connect error";

#[derive(Debug, Default)]
pub(crate) struct GRPCResult {
    // The mDNS address queried (None if mDNS was not used in connection establishment).
    mdns_address: Option<SocketAddr>,
    // The time taken to query mDNS (None if mDNS was not used in connection establishment or
    // query failed).
    mdns_query: Option<Duration>,

    // The time taken to complete authentication (None if authentication was unsuccessful).
    authentication: Option<Duration>,

    // The time taken to establish a connection (None if connection establishment was
    // unsuccessful).
    connection: Option<Duration>,

    // An error message possibly returned by dial's `connect` method (None if connection
    // establishment was successful).
    dial_error_message: Option<String>,
}

impl fmt::Display for GRPCResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(a) = self.mdns_address {
            writeln!(f, "mDNS address {} was used for connection", a)?;
        }
        match self.mdns_query {
            Some(d) => {
                writeln!(f, "mDNS queried in {}ms", d.num_milliseconds(),)?;
            }
            None => {
                writeln!(f, "mDNS could not be used to connect")?;
            }
        }

        match self.authentication {
            Some(d) => {
                writeln!(f, "authentication successful in {}ms", d.num_milliseconds(),)?;
            }
            None => {
                writeln!(f, "authentication failed")?;
            }
        }

        match self.connection {
            Some(d) => {
                writeln!(
                    f,
                    "gRPC connection establishment successful in {}ms",
                    d.num_milliseconds(),
                )?;
            }
            None => {
                writeln!(f, "gRPC connection establishment failed")?;
            }
        }

        if let Some(emsg) = &self.dial_error_message {
            writeln!(f, "\n{emsg}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub(crate) struct WebRTCResult {
    // The mDNS address queried (None if mDNS was not used in connection establishment).
    mdns_address: Option<SocketAddr>,
    // The time taken to query mDNS (None if mDNS was not used in connection establishment or
    // query failed).
    mdns_query: Option<Duration>,

    // The time taken to complete authentication (None if authentication was unsuccessful).
    authentication: Option<Duration>,

    // An error message possibly returned by dial's `connect` method (None if connection
    // establishment was successful).
    dial_error_message: Option<String>,

    // The stringified selected candidate pair for the connection (None if connection establishment
    // was unsuccessful).
    selected_candidate_pair: Option<String>,

    // The time taken to establish a connection (None if connection establishment was
    // unsuccessful).
    connection: Option<Duration>,
}

impl fmt::Display for WebRTCResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(a) = self.mdns_address {
            writeln!(f, "mDNS address {} was used for connection", a)?;
        }
        match self.mdns_query {
            Some(d) => {
                writeln!(f, "mDNS queried in {}ms", d.num_milliseconds(),)?;
            }
            None => {
                writeln!(f, "mDNS could not be used to connect")?;
            }
        }

        match self.authentication {
            Some(d) => {
                writeln!(f, "authentication successful in {}ms", d.num_milliseconds(),)?;
            }
            None => {
                writeln!(f, "authentication failed")?;
            }
        }

        match self.connection {
            Some(d) => {
                writeln!(
                    f,
                    "WebRTC connection establishment successful in {}ms",
                    d.num_milliseconds(),
                )?;
            }
            None => {
                writeln!(f, "WebRTC connection establishment failed")?;
            }
        }

        if let Some(emsg) = &self.dial_error_message {
            writeln!(f, "\n{emsg}")?;
        }

        if let Some(c) = &self.selected_candidate_pair {
            writeln!(f, "selected ICE candidate pair was:\n\t{c}")?;
        }

        Ok(())
    }
}

fn extract_timestamp(log: &str) -> Result<DateTime<FixedOffset>> {
    let split_log = log.split_whitespace().collect::<Vec<&str>>();
    if split_log.len() == 0 {
        bail!("malformed log returned by dial: {log}");
    }
    match DateTime::parse_from_rfc3339(split_log[0]) {
        Ok(d) => Ok(d),
        Err(e) => bail!("error parsing timestamp in log {log}: {e}"),
    }
}

fn extract_mdns_address(log: &str) -> Result<SocketAddr> {
    let mut split_log = log.split_whitespace().collect::<Vec<&str>>();

    // mDNS IP address should be last token in log.
    match split_log.pop() {
        Some(a) => match a.parse::<SocketAddr>() {
            Ok(a) => Ok(a),
            Err(e) => bail!("error parsing IP address {a} in log {log}: {e}"),
        },
        None => bail!("malformed mDNS log returned by dial: {log}"),
    }
}

fn extract_dial_error(log: &str) -> Result<String> {
    // Tear off LOG prefixes and reattach the DIAL_ERROR_PREFIX.
    let split_log = log.split(DIAL_ERROR_PREFIX).collect::<Vec<&str>>();
    if split_log.len() != 2 {
        bail!("malformed dial error message: {log}");
    }
    Ok(format!("{}{}", DIAL_ERROR_PREFIX, split_log[1]))
}

pub(crate) fn parse_grpc_logs(
    log_path: PathBuf,
    out: &mut Box<dyn io::Write>,
) -> Result<GRPCResult> {
    let mut res = GRPCResult::default();

    let mut connection_establishment_start = None;
    let mut authentication_start = None;
    let mut mdns_query_start = None;
    for log in fs::read_to_string(log_path)?.lines() {
        // Write actual log if in development mode.
        if DEVELOPMENT.is_some() {
            writeln!(out, "log message: {log}")?;
        }

        if log.contains(DIAL_ERROR_PREFIX) {
            res.dial_error_message = Some(extract_dial_error(log)?);
        } else if log.contains(log_prefixes::MDNS_QUERY_ATTEMPT) {
            mdns_query_start = Some(extract_timestamp(log)?);
        } else if log.contains(log_prefixes::MDNS_ADDRESS_FOUND) {
            match mdns_query_start {
                Some(mqs) => {
                    res.mdns_query = Some(extract_timestamp(log)?.signed_duration_since(mqs));
                }
                None => {
                    bail!(
                        "expected '{}' log before '{}'",
                        log_prefixes::MDNS_QUERY_ATTEMPT,
                        log_prefixes::MDNS_ADDRESS_FOUND
                    );
                }
            }
            res.mdns_address = Some(extract_mdns_address(log)?);
        } else if log.contains(log_prefixes::ACQUIRING_AUTH_TOKEN) {
            authentication_start = Some(extract_timestamp(log)?);
        } else if log.contains(log_prefixes::ACQUIRED_AUTH_TOKEN) {
            match authentication_start {
                Some(aus) => {
                    res.authentication = Some(extract_timestamp(log)?.signed_duration_since(aus));
                }
                None => {
                    bail!(
                        "expected '{}' log before '{}'",
                        log_prefixes::ACQUIRING_AUTH_TOKEN,
                        log_prefixes::ACQUIRED_AUTH_TOKEN
                    );
                }
            }
        } else if log.contains(log_prefixes::DIAL_ATTEMPT) {
            connection_establishment_start = Some(extract_timestamp(log)?);
        } else if log.contains(log_prefixes::DIALED_GRPC) {
            match connection_establishment_start {
                Some(ces) => {
                    res.connection = Some(extract_timestamp(log)?.signed_duration_since(ces));
                }
                None => {
                    bail!(
                        "expected '{}' log before '{}'",
                        log_prefixes::DIAL_ATTEMPT,
                        log_prefixes::DIALED_GRPC
                    );
                }
            }
        }
    }

    Ok(res)
}

fn extract_ice_candidate_pair(log: &str) -> Result<String> {
    // Tear off LOG prefixes.
    let split_log = log
        .split(log_prefixes::CANDIDATE_SELECTED)
        .collect::<Vec<&str>>();
    if split_log.len() != 2 {
        bail!("malformed selected candidate message: {log}");
    }

    // Remove annoying ": " still left over from log.
    Ok(split_log[1]
        .strip_prefix(": ")
        .unwrap_or_default()
        .to_string())
}

pub(crate) fn parse_webrtc_logs(
    log_path: PathBuf,
    out: &mut Box<dyn io::Write>,
) -> Result<WebRTCResult> {
    let mut res = WebRTCResult::default();

    let mut connection_establishment_start = None;
    let mut authentication_start = None;
    let mut mdns_query_start = None;
    for log in fs::read_to_string(log_path)?.lines() {
        // Write actual log if in development mode.
        if DEVELOPMENT.is_some() {
            writeln!(out, "log message: {log}")?;
        }

        if log.contains(DIAL_ERROR_PREFIX) {
            res.dial_error_message = Some(extract_dial_error(log)?);
        } else if log.contains(log_prefixes::MDNS_QUERY_ATTEMPT) {
            mdns_query_start = Some(extract_timestamp(log)?);
        } else if log.contains(log_prefixes::MDNS_ADDRESS_FOUND) {
            match mdns_query_start {
                Some(mqs) => {
                    res.mdns_query = Some(extract_timestamp(log)?.signed_duration_since(mqs));
                }
                None => {
                    bail!(
                        "expected '{}' log before '{}'",
                        log_prefixes::MDNS_QUERY_ATTEMPT,
                        log_prefixes::MDNS_ADDRESS_FOUND
                    );
                }
            }
            res.mdns_address = Some(extract_mdns_address(log)?);
        } else if log.contains(log_prefixes::ACQUIRING_AUTH_TOKEN) {
            authentication_start = Some(extract_timestamp(log)?);
        } else if log.contains(log_prefixes::ACQUIRED_AUTH_TOKEN) {
            match authentication_start {
                Some(aus) => {
                    res.authentication = Some(extract_timestamp(log)?.signed_duration_since(aus));
                }
                None => {
                    bail!(
                        "expected '{}' log before '{}'",
                        log_prefixes::ACQUIRING_AUTH_TOKEN,
                        log_prefixes::ACQUIRED_AUTH_TOKEN
                    );
                }
            }
        } else if log.contains(log_prefixes::CANDIDATE_SELECTED) {
            res.selected_candidate_pair = Some(extract_ice_candidate_pair(log)?);
        } else if log.contains(log_prefixes::DIAL_ATTEMPT) {
            connection_establishment_start = Some(extract_timestamp(log)?);
        } else if log.contains(log_prefixes::DIALED_WEBRTC) {
            match connection_establishment_start {
                Some(ces) => {
                    res.connection = Some(extract_timestamp(log)?.signed_duration_since(ces));
                }
                None => {
                    bail!(
                        "expected '{}' log before '{}'",
                        log_prefixes::DIAL_ATTEMPT,
                        log_prefixes::DIALED_WEBRTC
                    );
                }
            }
        }
    }

    Ok(res)
}
