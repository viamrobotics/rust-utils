//! Best-effort reporting of WebRTC dial outcomes to the signaling server, mirroring the Go
//! SDK's connection-metadata telemetry (goutils rpc/wrtc_client_report.go). After a WebRTC
//! dial finishes — success or failure — the dialing client reports how far the dial got, how
//! it was signaled, how long it took, and (on success) which ICE candidate pair was selected.

use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use ::http::{HeaderValue, Uri};
use ::webrtc::ice::candidate::{CandidatePairState, CandidateType};
use ::webrtc::peer_connection::RTCPeerConnection;
use ::webrtc::stats::{StatsReport, StatsReportType};
use tonic::transport::Channel;
use tower_http::auth::AddAuthorization;
use tower_http::set_header::SetRequestHeader;

use crate::gen::proto::rpc::webrtc::v1::{
    signaling_service_client::SignalingServiceClient, ConnectionCandidate, ConnectionSignalingPath,
    DialStage, IceCandidateType, ReportConnectionMetadataRequest, SdkType,
};

/// How long a best-effort report send may take before being abandoned.
const REPORT_TIMEOUT: Duration = Duration::from_secs(5);

/// DialStageTracker tracks the furthest checkpoint a WebRTC dial reached, so a failed dial can
/// report where it stopped. It is advanced from the dial task and from candidate-exchange / ICE
/// callbacks, so it is an atomic; advance only ever moves it forward.
#[derive(Debug, Default)]
pub(crate) struct DialStageTracker {
    stage: AtomicI32,
}

impl DialStageTracker {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn advance(&self, stage: DialStage) {
        let mut cur = self.stage.load(Ordering::Acquire);
        while (stage as i32) > cur {
            match self.stage.compare_exchange_weak(
                cur,
                stage as i32,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return,
                Err(actual) => cur = actual,
            }
        }
    }

    pub(crate) fn reached(&self) -> DialStage {
        DialStage::from_i32(self.stage.load(Ordering::Acquire)).unwrap_or(DialStage::Unspecified)
    }
}

/// The Viam app signaling server hosts.
const VIAM_CLOUD_SIGNALING_HOSTS: [&str; 2] = ["app.viam.com", "app.viam.dev"];

/// Derives how a connection was signaled from the signaling server address: a Viam app
/// signaling host is CLOUD_SIGNALED; everything else (localhost, private/LAN addresses, a
/// machine's own signaling server, etc.) is LOCAL. WebRTC is never dialed over an
/// mDNS-discovered path here (mDNS connections are direct gRPC), so MDNS_LOCAL is never
/// reported by this SDK.
pub(crate) fn classify_signaling_path(signaling_uri: &Uri) -> ConnectionSignalingPath {
    let host = signaling_uri.host().unwrap_or_default().to_lowercase();
    if VIAM_CLOUD_SIGNALING_HOSTS.contains(&host.as_str()) {
        ConnectionSignalingPath::CloudSignaled
    } else {
        ConnectionSignalingPath::Local
    }
}

/// Extracts the gRPC status code of a failed dial: the code of the first tonic status in the
/// error chain, or Unknown when the failure was not a gRPC error.
pub(crate) fn failure_code(err: &anyhow::Error) -> i32 {
    for cause in err.chain() {
        if let Some(status) = cause.downcast_ref::<tonic::Status>() {
            return status.code() as i32;
        }
    }
    tonic::Code::Unknown as i32
}

/// Inspects the selected ICE candidate pair and classifies each side into a
/// ConnectionCandidate. Both are UNSPECIFIED when no succeeded, nominated pair exists.
///
/// We match on the nominated, succeeded pair rather than the transport's authoritative
/// selected pair because webrtc-rs's get_selected_candidate_pair returns an opaque
/// RTCIceCandidatePair (no accessors for its candidates). This is reliable here: webrtc-rs
/// uses regular nomination (a single nominated pair) and we sample once at dial completion.
pub(crate) fn classify_connection(
    stats: &StatsReport,
) -> (ConnectionCandidate, ConnectionCandidate) {
    let pair = stats.reports.values().find_map(|s| {
        if let StatsReportType::CandidatePair(p) = s {
            if p.nominated && p.state == CandidatePairState::Succeeded {
                return Some((p.local_candidate_id.clone(), p.remote_candidate_id.clone()));
            }
        }
        None
    });
    let (local_id, remote_id) = pair.unwrap_or_default();
    (
        classify_candidate(stats, &local_id),
        classify_candidate(stats, &remote_id),
    )
}

/// Maps a single ICE candidate stat to a ConnectionCandidate; a missing or unrecognized
/// candidate yields type UNSPECIFIED. Relay candidates carry the relay server address so the
/// signaling server can classify the relay provider.
fn classify_candidate(stats: &StatsReport, cand_id: &str) -> ConnectionCandidate {
    let cand = stats.reports.get(cand_id).and_then(|s| match s {
        StatsReportType::LocalCandidate(c) | StatsReportType::RemoteCandidate(c) => Some(c),
        _ => None,
    });
    match cand {
        Some(c) => match c.candidate_type {
            CandidateType::Host => ConnectionCandidate {
                r#type: IceCandidateType::Host as i32,
                relay_address: String::new(),
            },
            CandidateType::ServerReflexive | CandidateType::PeerReflexive => ConnectionCandidate {
                r#type: IceCandidateType::Stun as i32,
                relay_address: String::new(),
            },
            CandidateType::Relay => ConnectionCandidate {
                r#type: IceCandidateType::Relay as i32,
                relay_address: c.ip.clone(),
            },
            CandidateType::Unspecified => ConnectionCandidate::default(),
        },
        None => ConnectionCandidate::default(),
    }
}

/// Reports the outcome of one WebRTC dial to the signaling server it dialed through,
/// best-effort: errors are logged, never surfaced. The send is spawned on a detached task with
/// its own timeout so a failed dial's direct-connection fallback is not delayed and a report
/// still goes out if the caller moves on. An Unimplemented failure means the remote has no
/// WebRTC signaler — a fallback control flow, not a WebRTC failure — so it is never reported.
pub(crate) fn report_dial_outcome(
    channel: AddAuthorization<SetRequestHeader<Channel, HeaderValue>>,
    peer_connection: Option<Arc<RTCPeerConnection>>,
    stage: &DialStageTracker,
    dial_start: Instant,
    signaling_path: ConnectionSignalingPath,
    failure: Option<&anyhow::Error>,
) {
    let code = match failure {
        Some(err) => {
            let code = failure_code(err);
            if code == tonic::Code::Unimplemented as i32 {
                return;
            }
            code
        }
        None => 0,
    };
    let reached_stage = stage.reached() as i32;
    let duration_ms = u32::try_from(dial_start.elapsed().as_millis()).unwrap_or(u32::MAX);

    tokio::spawn(async move {
        let (local, remote) = match peer_connection {
            Some(pc) => classify_connection(&pc.get_stats().await),
            None => Default::default(),
        };
        let request = ReportConnectionMetadataRequest {
            local: Some(local),
            remote: Some(remote),
            sdk_type: SdkType::PythonCpp as i32,
            reached_stage,
            duration_ms,
            signaling_path: signaling_path as i32,
            failure_code: code,
            sdk_version: env!("CARGO_PKG_VERSION").to_string(),
        };
        let mut signaling_client = SignalingServiceClient::new(channel);
        match tokio::time::timeout(
            REPORT_TIMEOUT,
            signaling_client.report_connection_metadata(request),
        )
        .await
        {
            Ok(Err(e)) => log::debug!("failed to report connection metadata: {e}"),
            Err(_) => log::debug!("timed out reporting connection metadata"),
            Ok(Ok(_)) => (),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::webrtc::stats::{ICECandidatePairStats, ICECandidateStats, RTCStatsType};
    use std::collections::HashMap;
    use tokio::time::Instant;

    fn candidate_stats(id: &str, candidate_type: CandidateType, ip: &str) -> ICECandidateStats {
        ICECandidateStats {
            timestamp: Instant::now(),
            stats_type: RTCStatsType::LocalCandidate,
            id: id.to_string(),
            candidate_type,
            deleted: false,
            ip: ip.to_string(),
            network_type: Default::default(),
            port: 0,
            priority: 0,
            relay_protocol: String::new(),
            url: String::new(),
        }
    }

    fn pair_stats(
        local_id: &str,
        remote_id: &str,
        state: CandidatePairState,
        nominated: bool,
    ) -> ICECandidatePairStats {
        ICECandidatePairStats {
            timestamp: Instant::now(),
            stats_type: RTCStatsType::CandidatePair,
            id: format!("{local_id}-{remote_id}"),
            local_candidate_id: local_id.to_string(),
            remote_candidate_id: remote_id.to_string(),
            state,
            nominated,
            packets_sent: 0,
            packets_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            last_packet_sent_timestamp: Instant::now(),
            last_packet_received_timestamp: Instant::now(),
            total_round_trip_time: 0.0,
            current_round_trip_time: 0.0,
            available_outgoing_bitrate: 0.0,
            available_incoming_bitrate: 0.0,
            requests_received: 0,
            requests_sent: 0,
            responses_received: 0,
            responses_sent: 0,
            consent_requests_sent: 0,
            circuit_breaker_trigger_count: 0,
            consent_expired_timestamp: Instant::now(),
            first_request_timestamp: Instant::now(),
            last_request_timestamp: Instant::now(),
            retransmissions_sent: 0,
        }
    }

    fn report(entries: Vec<(String, StatsReportType)>) -> StatsReport {
        StatsReport {
            reports: entries.into_iter().collect::<HashMap<_, _>>(),
        }
    }

    #[test]
    fn test_dial_stage_tracker_only_advances_forward() {
        let tracker = DialStageTracker::new();
        assert_eq!(tracker.reached(), DialStage::Unspecified);
        tracker.advance(DialStage::ConfigFetched);
        assert_eq!(tracker.reached(), DialStage::ConfigFetched);
        tracker.advance(DialStage::SignalingConnected);
        assert_eq!(tracker.reached(), DialStage::ConfigFetched);
        tracker.advance(DialStage::Ready);
        assert_eq!(tracker.reached(), DialStage::Ready);
    }

    #[test]
    fn test_classify_signaling_path() {
        for (uri, expected) in [
            (
                "https://app.viam.com:443",
                ConnectionSignalingPath::CloudSignaled,
            ),
            (
                "https://app.viam.dev",
                ConnectionSignalingPath::CloudSignaled,
            ),
            (
                "https://APP.VIAM.COM:443",
                ConnectionSignalingPath::CloudSignaled,
            ),
            ("http://localhost:8080", ConnectionSignalingPath::Local),
            ("http://127.0.0.1:9000", ConnectionSignalingPath::Local),
            ("http://10.1.2.3:443", ConnectionSignalingPath::Local),
            (
                "https://my-robot.abc123.viam.cloud:443",
                ConnectionSignalingPath::Local,
            ),
        ] {
            let uri: Uri = uri.parse().unwrap();
            assert_eq!(classify_signaling_path(&uri), expected, "uri: {uri}");
        }
    }

    #[test]
    fn test_failure_code() {
        let status_err =
            anyhow::Error::from(tonic::Status::not_found("robot offline")).context("dial failed");
        assert_eq!(failure_code(&status_err), tonic::Code::NotFound as i32);

        let plain_err = anyhow::anyhow!("timed out opening data channel");
        assert_eq!(failure_code(&plain_err), tonic::Code::Unknown as i32);
    }

    #[test]
    fn test_classify_connection_selects_nominated_succeeded_pair() {
        let stats = report(vec![
            (
                "local-relay".to_string(),
                StatsReportType::LocalCandidate(candidate_stats(
                    "local-relay",
                    CandidateType::Relay,
                    "34.0.0.1",
                )),
            ),
            (
                "remote-host".to_string(),
                StatsReportType::RemoteCandidate(candidate_stats(
                    "remote-host",
                    CandidateType::Host,
                    "192.168.1.2",
                )),
            ),
            (
                "pair-failed".to_string(),
                StatsReportType::CandidatePair(pair_stats(
                    "other",
                    "other",
                    CandidatePairState::Failed,
                    false,
                )),
            ),
            (
                "pair-selected".to_string(),
                StatsReportType::CandidatePair(pair_stats(
                    "local-relay",
                    "remote-host",
                    CandidatePairState::Succeeded,
                    true,
                )),
            ),
        ]);

        let (local, remote) = classify_connection(&stats);
        assert_eq!(local.r#type, IceCandidateType::Relay as i32);
        assert_eq!(local.relay_address, "34.0.0.1");
        assert_eq!(remote.r#type, IceCandidateType::Host as i32);
        assert_eq!(remote.relay_address, "");
    }

    #[test]
    fn test_classify_connection_no_selected_pair_is_unspecified() {
        let stats = report(vec![(
            "pair-waiting".to_string(),
            StatsReportType::CandidatePair(pair_stats(
                "a",
                "b",
                CandidatePairState::Waiting,
                false,
            )),
        )]);
        let (local, remote) = classify_connection(&stats);
        assert_eq!(local.r#type, IceCandidateType::Unspecified as i32);
        assert_eq!(remote.r#type, IceCandidateType::Unspecified as i32);
    }

    #[test]
    fn test_classify_candidate_types() {
        for (candidate_type, expected, expected_addr) in [
            (CandidateType::Host, IceCandidateType::Host, ""),
            (CandidateType::ServerReflexive, IceCandidateType::Stun, ""),
            (CandidateType::PeerReflexive, IceCandidateType::Stun, ""),
            (CandidateType::Relay, IceCandidateType::Relay, "5.6.7.8"),
        ] {
            let stats = report(vec![(
                "c1".to_string(),
                StatsReportType::LocalCandidate(candidate_stats("c1", candidate_type, "5.6.7.8")),
            )]);
            let got = classify_candidate(&stats, "c1");
            assert_eq!(got.r#type, expected as i32, "type: {candidate_type}");
            assert_eq!(got.relay_address, expected_addr, "type: {candidate_type}");
        }

        let stats = report(vec![]);
        let got = classify_candidate(&stats, "missing");
        assert_eq!(got.r#type, IceCandidateType::Unspecified as i32);
    }
}
