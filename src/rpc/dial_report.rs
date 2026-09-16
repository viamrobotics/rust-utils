//! Best-effort reporting of WebRTC dial connection metadata to the Viam app signaling server.
//!
//! This is a port of goutils#583 (`rpc/wrtc_client_report.go`). After a WebRTC dial finishes —
//! success or failure — the client reports to the app signaling server it dialed through: the
//! furthest dial stage reached, the gRPC failure code, the dial duration, how the dial was
//! signaled, and the selected ICE candidate pair per side (host / stun / relay, plus the relay
//! address). Delivery is best-effort and runs in a detached background task so reporting never
//! adds latency to, or fails, a dial.
//!
//! Only the app signaling server implements `ReportConnectionMetadata`. A cloud-signaled dial
//! (routed through `app.viam.com` / `app.viam.dev`) reports over its own channel; a locally-signaled
//! dial reconstructs an authenticated connection to prod app, reusing the dial's credentials (Go's
//! `fixUpReportDialOpts`). rust-utils does not signal WebRTC over mDNS, so no report carries the
//! `MdnsLocal` path. See [`should_deliver`] for which outcomes are reported.

use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;

use ::http::HeaderValue;
use ::webrtc::ice::candidate::{CandidatePairState, CandidateType};
use ::webrtc::peer_connection::RTCPeerConnection;
use ::webrtc::stats::{StatsReport, StatsReportType};
use tonic::transport::Channel;
use tower_http::auth::AddAuthorization;
use tower_http::set_header::SetRequestHeader;

use crate::gen::proto::rpc::webrtc::v1::{
    signaling_service_client::SignalingServiceClient, ConnectionCandidate, ConnectionSignalingPath,
    DialStage, IceCandidateType, ReportConnectionMetadataRequest,
};

/// The authenticated, rpc-host-stamped channel used to reach a signaling server, over which a
/// report can be delivered.
type SignalingChannel = AddAuthorization<SetRequestHeader<Channel, HeaderValue>>;

/// gRPC status code reported for a failed dial whose error is not a `tonic::Status`. Matches Go's
/// `status.Code` returning `codes.Unknown` for non-status errors.
const STATUS_CODE_UNKNOWN: i32 = 2;

/// How long to wait for a report RPC before giving up.
const REPORT_TIMEOUT: Duration = Duration::from_secs(5);

/// The Viam app signaling server hosts. A dial signaled through one of these is "cloud-signaled".
const VIAM_CLOUD_SIGNALING_HOSTS: [&str; 2] = ["app.viam.com", "app.viam.dev"];

/// Tracks the furthest dial checkpoint a WebRTC dial reached. It is advanced from several tasks
/// (the dial goroutine, the candidate-exchange task, and ICE/peer-connection callbacks), so it is
/// an atomic; `advance` only ever moves it forward.
pub(crate) struct StageTracker(AtomicI32);

impl StageTracker {
    pub(crate) fn new() -> Self {
        StageTracker(AtomicI32::new(DialStage::Unspecified as i32))
    }

    /// Moves the reached stage forward to `stage` if `stage` is further than the current value.
    pub(crate) fn advance(&self, stage: DialStage) {
        let next = stage as i32;
        let mut cur = self.0.load(Ordering::Acquire);
        while next > cur {
            match self
                .0
                .compare_exchange_weak(cur, next, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => break,
                Err(actual) => cur = actual,
            }
        }
    }

    /// The furthest stage reached so far, as the raw enum value.
    pub(crate) fn reached(&self) -> i32 {
        self.0.load(Ordering::Acquire)
    }
}

/// The env var that opts a process out of dial reporting (any non-empty value disables it).
const DISABLE_DIAL_REPORTING_ENV: &str = "VIAM_DISABLE_DIAL_REPORTING";

/// Whether a dial should deliver its connection report.
///
/// Disabled under `cfg(test)` (the crate's own unit tests) and whenever `VIAM_DISABLE_DIAL_REPORTING`
/// is set to a non-empty value, so a detached report task can't outlive a test (mirrors Go
/// disabling reports in test binaries). `cfg(test)` does not cover integration tests in `tests/` or
/// downstream consumers, so test/CI harnesses that dial real robots should set the env var;
/// production consumers (release builds, no env var) report normally.
pub(crate) fn dial_reporting_enabled() -> bool {
    if cfg!(test) {
        return false;
    }
    let disabled = std::env::var(DISABLE_DIAL_REPORTING_ENV)
        .map(|value| !value.is_empty())
        .unwrap_or(false);
    !disabled
}

/// Derives how a WebRTC dial was signaled from the signaling server host. rust-utils only signals
/// WebRTC over the cloud or a local signaling server (it does not yet support WebRTC over mDNS),
/// so this returns `CloudSignaled` or `Local` — never `MdnsLocal`.
pub(crate) fn classify_signaling_path(signaling_host: &str) -> ConnectionSignalingPath {
    // Strip a trailing :port if present. (Signaling hosts are never bracketed IPv6, so a plain
    // rsplit on ':' is safe.)
    let host = signaling_host
        .rsplit_once(':')
        .map_or(signaling_host, |(host, _port)| host);
    if VIAM_CLOUD_SIGNALING_HOSTS.contains(&host.to_ascii_lowercase().as_str()) {
        ConnectionSignalingPath::CloudSignaled
    } else {
        ConnectionSignalingPath::Local
    }
}

/// Returns the (local, remote) candidate ids of the ICE candidate pair a WebRTC connection settled
/// on — the nominated pair in the succeeded state — or None if no such pair exists.
fn selected_candidate_pair(stats: &StatsReport) -> Option<(String, String)> {
    stats.reports.values().find_map(|report| match report {
        StatsReportType::CandidatePair(pair)
            if pair.nominated && pair.state == CandidatePairState::Succeeded =>
        {
            Some((
                pair.local_candidate_id.clone(),
                pair.remote_candidate_id.clone(),
            ))
        }
        _ => None,
    })
}

/// Inspects the selected ICE candidate pair and classifies each side into a `ConnectionCandidate`.
/// Both are the default (type UNSPECIFIED) when `peer` is None (a failed dial) or no succeeded,
/// nominated pair exists.
pub(crate) async fn classify_connection(
    peer: Option<&RTCPeerConnection>,
) -> (ConnectionCandidate, ConnectionCandidate) {
    let Some(peer) = peer else {
        return (
            ConnectionCandidate::default(),
            ConnectionCandidate::default(),
        );
    };
    let stats = peer.get_stats().await;
    let Some((local_id, remote_id)) = selected_candidate_pair(&stats) else {
        return (
            ConnectionCandidate::default(),
            ConnectionCandidate::default(),
        );
    };
    (
        classify_candidate(&stats, &local_id),
        classify_candidate(&stats, &remote_id),
    )
}

/// Maps a single ICE candidate stat to a `ConnectionCandidate`; a missing or unrecognized
/// candidate yields type UNSPECIFIED. Relay candidates carry the relay server address so the
/// signaling server can classify the relay provider.
fn classify_candidate(stats: &StatsReport, candidate_id: &str) -> ConnectionCandidate {
    let candidate = match stats.reports.get(candidate_id) {
        Some(StatsReportType::LocalCandidate(candidate))
        | Some(StatsReportType::RemoteCandidate(candidate)) => candidate,
        _ => return ConnectionCandidate::default(),
    };
    match candidate.candidate_type {
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
            relay_address: candidate.ip.clone(),
        },
        CandidateType::Unspecified => ConnectionCandidate::default(),
    }
}

/// The gRPC status code to report for a failed dial: the `tonic::Status` code if the error chain
/// carries one, otherwise UNKNOWN (matching Go's `status.Code` for non-status errors).
pub(crate) fn failure_code(err: &anyhow::Error) -> i32 {
    for cause in err.chain() {
        if let Some(status) = cause.downcast_ref::<tonic::Status>() {
            return status.code() as i32;
        }
    }
    STATUS_CODE_UNKNOWN
}

/// Whether a built report should actually be delivered.
///
/// On a successful WebRTC dial only a READY report is truthful (a non-READY furthest stage on
/// success would count a spurious failure against a dial that succeeded).
///
/// A failed WebRTC dial is reported only when the failure is terminal — i.e. there is no working
/// fallback. rust-utils falls back to a direct gRPC connection on any WebRTC failure, so:
/// - **cloud-signaled**: direct gRPC cannot reach a cloud robot, so the WebRTC failure fails the
///   dial and is reported;
/// - **local-signaled**: the direct gRPC connection to the robot generally succeeds, so the WebRTC
///   failure did not actually fail the dial — it is suppressed (mirrors Go suppressing a non-READY
///   report when the logical dial nonetheless succeeded).
pub(crate) fn should_deliver(
    reached_stage: i32,
    dial_succeeded: bool,
    signaling_path: ConnectionSignalingPath,
) -> bool {
    if dial_succeeded {
        reached_stage == DialStage::Ready as i32
    } else {
        signaling_path == ConnectionSignalingPath::CloudSignaled
    }
}

/// Delivers a single connection report over the (already authenticated, rpc-host-stamped) signaling
/// channel the dial used. Best-effort with a short timeout; failures are logged at debug and
/// otherwise swallowed. Intended to be spawned as a detached background task.
pub(crate) async fn send_dial_report(
    channel: SignalingChannel,
    request: ReportConnectionMetadataRequest,
) {
    let reached_stage = request.reached_stage;
    let mut client = SignalingServiceClient::new(channel);
    match tokio::time::timeout(REPORT_TIMEOUT, client.report_connection_metadata(request)).await {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => {
            log::debug!("failed to report connection metadata (reached_stage={reached_stage}): {e}")
        }
        Err(_) => {
            log::debug!("timed out reporting connection metadata (reached_stage={reached_stage})")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_signaling_path_cloud_vs_local() {
        assert_eq!(
            classify_signaling_path("app.viam.com:443"),
            ConnectionSignalingPath::CloudSignaled
        );
        assert_eq!(
            classify_signaling_path("app.viam.com"),
            ConnectionSignalingPath::CloudSignaled
        );
        assert_eq!(
            classify_signaling_path("APP.VIAM.DEV:443"),
            ConnectionSignalingPath::CloudSignaled
        );
        assert_eq!(
            classify_signaling_path("app.viaminternal:8089"),
            ConnectionSignalingPath::Local
        );
        assert_eq!(
            classify_signaling_path("localhost:8080"),
            ConnectionSignalingPath::Local
        );
        assert_eq!(
            classify_signaling_path("10.1.2.3:443"),
            ConnectionSignalingPath::Local
        );
    }

    #[test]
    fn should_deliver_suppresses_non_ready_on_success() {
        use ConnectionSignalingPath::{CloudSignaled, Local};
        // Success: only a READY report is delivered, regardless of signaling path.
        assert!(should_deliver(DialStage::Ready as i32, true, CloudSignaled));
        assert!(should_deliver(DialStage::Ready as i32, true, Local));
        assert!(!should_deliver(
            DialStage::IceConnected as i32,
            true,
            CloudSignaled
        ));
        assert!(!should_deliver(DialStage::Unspecified as i32, true, Local));
        // Cloud failure: reported (direct gRPC can't reach a cloud robot, so it's terminal).
        assert!(should_deliver(
            DialStage::Unspecified as i32,
            false,
            CloudSignaled
        ));
        assert!(should_deliver(
            DialStage::OfferSent as i32,
            false,
            CloudSignaled
        ));
        // Local failure: suppressed (the dial falls back to a working direct gRPC connection).
        assert!(!should_deliver(DialStage::OfferSent as i32, false, Local));
        assert!(!should_deliver(
            DialStage::IceConnected as i32,
            false,
            Local
        ));
    }

    #[test]
    fn stage_tracker_only_moves_forward() {
        let tracker = StageTracker::new();
        assert_eq!(tracker.reached(), DialStage::Unspecified as i32);
        tracker.advance(DialStage::OfferSent);
        assert_eq!(tracker.reached(), DialStage::OfferSent as i32);
        // A lower stage does not regress the tracker.
        tracker.advance(DialStage::SignalingConnected);
        assert_eq!(tracker.reached(), DialStage::OfferSent as i32);
        tracker.advance(DialStage::Ready);
        assert_eq!(tracker.reached(), DialStage::Ready as i32);
    }

    #[test]
    fn failure_code_extracts_tonic_status() {
        let status = tonic::Status::new(tonic::Code::PermissionDenied, "nope");
        let err = anyhow::Error::from(status).context("while dialing");
        assert_eq!(failure_code(&err), tonic::Code::PermissionDenied as i32);

        let plain = anyhow::anyhow!("some non-status error");
        assert_eq!(failure_code(&plain), STATUS_CODE_UNKNOWN);
    }
}
