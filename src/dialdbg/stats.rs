use std::fmt;
use tokio::time::Instant;
use webrtc::stats;

pub(crate) struct StatsReport(pub(crate) stats::StatsReport);

impl fmt::Display for StatsReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // NOTE(benjirewis): StatsReport contains 13 types of stat reports; there may be more relevant stats
        // to print here, but for now I have stuck with only printing the candidates.
        writeln!(f, "\nnominated ICE candidates:\n")?;
        let now = Instant::now();
        for (_, value) in &self.0.reports {
            match value {
                stats::StatsReportType::LocalCandidate(ref cand)
                | stats::StatsReportType::RemoteCandidate(ref cand) => {
                    let remote_or_local = if let stats::StatsReportType::LocalCandidate(_) = value {
                        "local"
                    } else {
                        "remote"
                    };
                    writeln!(f, "\t{} ICE candidate:", remote_or_local)?;
                    writeln!(f, "\t\tIP address: {}", cand.ip)?;
                    writeln!(f, "\t\tport: {}", cand.port)?;
                    writeln!(
                        f,
                        "\t\tnominated {:#?} ago",
                        now.duration_since(cand.timestamp)
                    )?;
                    writeln!(f, "\t\trelay protocol: {}", cand.relay_protocol)?;
                    writeln!(f, "\t\tnetwork type: {}", cand.network_type)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
