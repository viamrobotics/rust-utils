use std::fmt;
use webrtc::stats;

pub(crate) struct StatsReport(pub(crate) stats::StatsReport);

impl fmt::Display for StatsReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // NOTE(benjirewis): StatsReport contains 13 types of stat reports; there may be more relevant stats
        // to print here, but for now I have stuck with only printing the candidates.
        writeln!(f, "\nnominated ICE candidate pair:\n")?;

        // Find the nominated candidate pair first, then look up its local/remote
        // candidates by ID so we only print the pair that was actually used.
        let nominated_pair = self.0.reports.values().find_map(|v| {
            if let stats::StatsReportType::CandidatePair(ref pair) = v {
                if pair.nominated {
                    return Some(pair);
                }
            }
            None
        });

        if let Some(pair) = nominated_pair {
            for (id, label) in [
                (&pair.local_candidate_id, "local"),
                (&pair.remote_candidate_id, "remote"),
            ] {
                let cand = self.0.reports.get(id).and_then(|v| match v {
                    stats::StatsReportType::LocalCandidate(c)
                    | stats::StatsReportType::RemoteCandidate(c) => Some(c),
                    _ => None,
                });
                let Some(cand) = cand else {
                    continue;
                };
                writeln!(f, "\t{label} ICE candidate:")?;
                writeln!(f, "\t\tIP address: {}", cand.ip)?;
                writeln!(f, "\t\tport: {}", cand.port)?;
                writeln!(f, "\t\tcandidate type: {}", cand.candidate_type)?;
                writeln!(f, "\t\tnominated {:#?} ago", cand.timestamp.elapsed())?;
                writeln!(f, "\t\trelay protocol: {}", cand.relay_protocol)?;
                writeln!(f, "\t\tnetwork type: {}", cand.network_type)?;
            }
        } else {
            writeln!(f, "\t(no nominated pair found)")?;
        }

        writeln!(f, "\nall ICE candidates:\n")?;
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
                    writeln!(f, "\t\tcandidate type: {}", cand.candidate_type)?;
                    writeln!(f, "\t\tnominated {:#?} ago", cand.timestamp.elapsed())?;
                    writeln!(f, "\t\trelay protocol: {}", cand.relay_protocol)?;
                    writeln!(f, "\t\tnetwork type: {}", cand.network_type)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
