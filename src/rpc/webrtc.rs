use super::log_prefixes;
use crate::gen::proto::rpc::webrtc::v1::{IceServer, ResponseTrailers, WebRtcConfig};
use anyhow::Result;
use bytes::Bytes;
use core::fmt;
use futures::Future;
use http::{header::HeaderName, HeaderMap, HeaderValue, Uri};
use std::{hint, str::FromStr, sync::Arc, time::Duration};
use webrtc::{
    api::{
        interceptor_registry, media_engine::MediaEngine, setting_engine::SettingEngine, APIBuilder,
        API,
    },
    data_channel::{
        data_channel_init::RTCDataChannelInit, data_channel_message::DataChannelMessage,
        RTCDataChannel,
    },
    dtls::extension::extension_use_srtp::SrtpProtectionProfile,
    ice::mdns::MulticastDnsMode,
    ice_transport::ice_server::RTCIceServer,
    interceptor::registry::Registry,
    peer_connection::{
        configuration::RTCConfiguration, peer_connection_state::RTCPeerConnectionState,
        policy::ice_transport_policy::RTCIceTransportPolicy,
        sdp::session_description::RTCSessionDescription, signaling_state::RTCSignalingState,
        RTCPeerConnection,
    },
};

// set to 20sec to match _defaultOfferDeadline in goutils/rpc/wrtc_call_queue.go
const WEBRTC_TIMEOUT: Duration = Duration::from_secs(20);

/// Options for connecting via webRTC.
#[derive(Default, Clone)]
pub(crate) struct Options {
    pub(crate) disable_webrtc: bool,
    pub(crate) disable_trickle_ice: bool,
    pub(crate) config: RTCConfiguration,
    pub(crate) signaling_insecure: bool,
    pub(crate) signaling_server_address: String,
    /// Forces ICE transport policy to relay-only, so only TURN candidates are used.
    /// Useful for testing relay connectivity through a TURN server.
    pub(crate) force_relay: bool,
    /// Strips TURN servers from the ICE configuration so only host and server-reflexive
    /// candidates are used. Useful for testing direct connectivity without relay fallback.
    pub(crate) force_p2p: bool,
    /// When set, filters the signaling server's TURN list to only the server whose
    /// parsed URI matches. Uses struct comparison identical to the server-side TURN_URI
    /// env var. Leave transport unspecified for UDP default.
    /// Example: "turn:turn.viam.com:443"
    pub(crate) turn_uri: Option<String>,

}

impl fmt::Debug for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Options")
            .field("disable_webrtc", &format_args!("{}", self.disable_webrtc))
            .field(
                "disable_trickle_ice",
                &format_args!("{}", self.disable_trickle_ice),
            )
            // RTCConfiguration does not derive Debug
            .field("config", &format_args!("{}", "<Opaque>"))
            .field(
                "signaling_insecure",
                &format_args!("{}", self.signaling_insecure),
            )
            .field(
                "signaling_server_address",
                &format_args!("{}", self.signaling_server_address),
            )
            .finish()
    }
}

impl Options {
    pub(crate) fn infer_signaling_server_address(uri: &Uri) -> Option<(String, bool)> {
        // TODO(RSDK-235): remove hard coding of signaling server address and prefer SRV lookup instead
        let path = uri.to_string();
        if path.contains(".viam.cloud") {
            Some(("app.viam.com:443".to_string(), true))
        } else if path.contains(".robot.viaminternal") {
            Some(("app.viaminternal:8089".to_string(), false))
        } else {
            None
        }
    }

    pub(crate) fn infer_from_uri(uri: Uri) -> Self {
        match Self::infer_signaling_server_address(&uri) {
            None => Options {
                config: default_configuration(),
                ..Default::default()
            },
            Some((signaling_server_address, secure)) => Options {
                config: default_configuration(),
                signaling_server_address,
                signaling_insecure: !secure,
                ..Default::default()
            },
        }
    }

    /// Disables connecting via webRTC, forcing a direct connect
    pub(crate) fn disable_webrtc(mut self) -> Self {
        self.disable_webrtc = true;
        self
    }

}

/// A parsed TURN URI with scheme, host, port, and transport components.
/// Transport defaults to "udp" when unspecified, matching stun.ParseURI behavior.
#[derive(Debug, PartialEq)]
pub(crate) struct TurnUri {
    pub scheme: String,
    pub host: String,
    pub port: u16,
    pub transport: String,
}

impl TurnUri {
    /// Parses a TURN URI string of the form "scheme:host:port?transport=proto".
    /// Returns None for non-TURN URIs or malformed input.
    pub fn parse(s: &str) -> Option<Self> {
        let (scheme, rest) = s.split_once(':')?;
        if scheme != "turn" && scheme != "turns" {
            return None;
        }
        let (hostport, query) = rest.split_once('?').unwrap_or((rest, ""));
        let (host, port_str) = hostport.rsplit_once(':')?;
        let port = port_str.parse().ok()?;
        let transport = query
            .split('&')
            .find_map(|p| p.strip_prefix("transport="))
            .unwrap_or("udp")
            .to_string();
        Some(TurnUri {
            scheme: scheme.to_string(),
            host: host.to_string(),
            port,
            transport,
        })
    }


}

/// Filters TURN server URLs in config to only those whose parsed URI matches turn_uri.
/// Non-TURN URLs (e.g. stun:) are always kept unchanged.
pub(crate) fn apply_turn_options(
    mut config: RTCConfiguration,
    turn_uri: Option<&TurnUri>,
) -> RTCConfiguration {
    if turn_uri.is_none() {
        return config;
    }
    for server in &mut config.ice_servers {
        server.urls = server
            .urls
            .iter()
            .filter_map(|url| {
                if !url.starts_with("turn:") && !url.starts_with("turns:") {
                    return Some(url.clone());
                }
                let uri = TurnUri::parse(url)?;
                if let Some(filter) = turn_uri {
                    if &uri != filter {
                        return None;
                    }
                }
                Some(url.clone())
            })
            .collect();
    }
    // Remove ICE server entries that had all their TURN URLs filtered out.
    config.ice_servers.retain(|s| !s.urls.is_empty());
    config
}

/// Returns true if any of the ICE server's URLs use a TURN scheme.
pub(crate) fn ice_server_has_turn(s: &RTCIceServer) -> bool {
    s.urls
        .iter()
        .any(|url| url.starts_with("turn:") || url.starts_with("turns:"))
}

/// Applies force_relay or force_p2p options to a config and optional server config.
pub(crate) fn apply_ice_policy(
    mut config: RTCConfiguration,
    mut optional: Option<WebRtcConfig>,
    force_relay: bool,
    force_p2p: bool,
) -> (RTCConfiguration, Option<WebRtcConfig>) {
    if force_p2p {
        optional = None;
        config.ice_servers.retain(|s| !ice_server_has_turn(s));
    }
    if force_relay {
        config.ice_transport_policy = RTCIceTransportPolicy::Relay;
    }
    (config, optional)
}

fn default_configuration() -> RTCConfiguration {
    let ice_server = RTCIceServer {
        urls: vec!["stun:global.stun.twilio.com:3478?transport=udp".to_string()],
        ..Default::default()
    };

    RTCConfiguration {
        ice_servers: vec![ice_server],
        ..Default::default()
    }
}

fn ice_server_from_proto(ice_server: IceServer) -> RTCIceServer {
    RTCIceServer {
        urls: ice_server.urls,
        username: ice_server.username,
        credential: ice_server.credential,
    }
}

pub(crate) fn extend_webrtc_config(
    original: RTCConfiguration,
    optional: Option<WebRtcConfig>,
) -> RTCConfiguration {
    match optional {
        None => original,
        Some(optional) => {
            let mut new_ice_servers = original.ice_servers;
            for additional_server in optional.additional_ice_servers {
                let additional_server = ice_server_from_proto(additional_server);
                new_ice_servers.push(additional_server);
            }

            RTCConfiguration {
                ice_servers: new_ice_servers,
                ..original
            }
        }
    }
}

fn new_webrtc_api() -> Result<API> {
    let mut media_engine = MediaEngine::default();
    media_engine.register_default_codecs()?;
    let registry = Registry::new();
    let interceptor =
        interceptor_registry::register_default_interceptors(registry, &mut media_engine)?;

    let mut setting_engine = SettingEngine::default();

    // A recent commit to the upstream webrtc library added `Srtp_Aead_Aes_256_Gcm` to the
    // list of default `SrtpProtectionProfile`s. This caused assertion failures upstream in
    // the `GenericArray` crate, which prevented us from connecting properly. Removing this
    // default (which is consistent with how `rust-utils` has operated for the past several
    // years) prevents the upstream conflicts and lets us avoid navigating potential conflicts
    // in reworking the upstream defaults.
    let srtp_protection_profiles = vec![
        SrtpProtectionProfile::Srtp_Aead_Aes_128_Gcm,
        SrtpProtectionProfile::Srtp_Aes128_Cm_Hmac_Sha1_80,
    ];
    setting_engine.set_srtp_protection_profiles(srtp_protection_profiles);
    setting_engine.set_ice_multicast_dns_mode(MulticastDnsMode::QueryAndGather);
    setting_engine.set_include_loopback_candidate(true);

    Ok(APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(interceptor)
        .with_setting_engine(setting_engine)
        .build())
}

fn create_invalid_sdp_err(err: serde_json::error::Error) -> webrtc::Error {
    webrtc::Error::Sdp(webrtc::sdp::Error::SdpInvalidValue(err.to_string()))
}

pub(crate) async fn new_peer_connection_for_client(
    config: RTCConfiguration,
    disable_trickle_ice: bool,
) -> Result<(Arc<RTCPeerConnection>, Arc<RTCDataChannel>)> {
    let web_api = new_webrtc_api()?;
    let peer_connection = Arc::new(web_api.new_peer_connection(config).await?);

    let data_channel_init = RTCDataChannelInit {
        negotiated: Some(0),
        ordered: Some(true),
        ..Default::default()
    };

    let negotiation_channel_init = RTCDataChannelInit {
        negotiated: Some(1),
        ordered: Some(true),
        ..Default::default()
    };

    peer_connection.on_peer_connection_state_change(Box::new(
        move |connection: RTCPeerConnectionState| {
            log::info!("peer connection state change: {connection}");
            if connection == RTCPeerConnectionState::Connected {
                log::debug!("{}", log_prefixes::DIALED_WEBRTC);
            }
            Box::pin(async move {})
        },
    ));

    peer_connection.on_signaling_state_change(Box::new(move |ssc: RTCSignalingState| {
        log::info!("new signaling state: {ssc}");
        Box::pin(async move {})
    }));

    let data_channel = peer_connection
        .create_data_channel("data", Some(data_channel_init))
        .await?;
    let negotiation_channel = peer_connection
        .create_data_channel("negotiation", Some(negotiation_channel_init))
        .await?;

    let nc = negotiation_channel.clone();
    let pc = Arc::downgrade(&peer_connection);

    negotiation_channel.on_message(Box::new(move |msg: DataChannelMessage| {
        let wpc = pc.clone();
        let nc = nc.clone();
        Box::pin(async move {
            let pc = match wpc.upgrade() {
                Some(pc) => pc,
                None => return,
            };
            let sdp_vec = msg.data.to_vec();
            let maybe_err = async move {
                let sdp = serde_json::from_slice::<RTCSessionDescription>(&sdp_vec)
                    .map_err(create_invalid_sdp_err)?;
                pc.set_remote_description(sdp).await?;
                let answer = pc.create_answer(None).await?;
                pc.set_local_description(answer).await?;
                let local_description = pc
                    .local_description()
                    .await
                    .ok_or("No local description set");
                let desc =
                    serde_json::to_vec(&local_description).map_err(create_invalid_sdp_err)?;
                let desc = Bytes::copy_from_slice(&desc);
                nc.send(&desc).await
            }
            .await;

            if let Err(e) = maybe_err {
                log::error!("Error processing sdp in negotiation channel: {e}");
            }
        })
    }));

    if disable_trickle_ice {
        let offer = peer_connection.create_offer(None).await?;
        let mut receiver = peer_connection.gathering_complete_promise().await;
        peer_connection.set_local_description(offer).await?;

        // TODO(RSDK-596): impl future here so we don't spin loop, which prevents this
        // from actually timing out.
        let promise_gathering_completed = async move {
            // Block until ICE gathering is complete since we signal back one complete SDP and
            // do not want to wait on trickle ice
            while receiver.recv().await.is_some() {
                hint::spin_loop();
            }
        };

        webrtc_action_with_timeout(promise_gathering_completed).await?;
    }

    Ok((peer_connection, data_channel))
}

pub(crate) async fn action_with_timeout<T>(
    f: impl Future<Output = T>,
    timeout: Duration,
) -> Result<T> {
    tokio::pin! {
        let timeout = tokio::time::sleep(timeout);
        let f = f;
    }

    tokio::select! {
        res = &mut f => {
            Ok(res)
        }
        _ = &mut timeout => {
            Err(anyhow::anyhow!("Action timed out"))
        }
    }
}

pub(crate) async fn webrtc_action_with_timeout<T>(f: impl Future<Output = T>) -> Result<T> {
    action_with_timeout(f, WEBRTC_TIMEOUT).await
}

pub(crate) fn trailers_from_proto(proto: ResponseTrailers) -> HeaderMap {
    let mut trailers = HeaderMap::new();
    if let Some(metadata) = proto.metadata {
        for (k, v) in metadata.md.iter() {
            let k = HeaderName::from_str(k);
            let v = HeaderValue::from_str(&v.values.concat());
            let (k, v) = match (k, v) {
                (Ok(k), Ok(v)) => (k, v),
                (Err(e), _) => {
                    log::error!("Error converting proto trailer key: [{e}]");
                    continue;
                }
                (_, Err(e)) => {
                    log::error!("Error converting proto trailer value: [{e}]");
                    continue;
                }
            };
            trailers.insert(k, v);
        }
    };

    let status_name = "grpc-status";
    let status_code = match proto.status {
        Some(ref status) => status.code.to_string(),
        None => "0".to_string(),
    };

    if let Some(ref status) = proto.status {
        let key = HeaderName::from_str("Grpc-Message");
        let val = HeaderValue::from_str(status.message.trim());
        match (key, val) {
            (Ok(k), Ok(v)) => {
                trailers.insert(k, v);
            }
            (Err(e), _) => log::error!("Error parsing HeaderName: {e}"),
            (_, Err(e)) => log::error!("Error parsing HeaderValue: {e}"),
        }
    }

    let k = match HeaderName::from_str(status_name) {
        Ok(k) => k,
        Err(e) => {
            log::error!("Error parsing HeaderName: {e}");
            return trailers;
        }
    };
    let v = match HeaderValue::from_str(&status_code) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Error parsing HeaderValue: {e}");
            return trailers;
        }
    };
    trailers.insert(k, v);
    trailers
}
