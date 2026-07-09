// @generated
/// A PacketMessage is used to packetize large messages (> 64KiB) to be able to safely
/// transmit over WebRTC data channels.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PacketMessage {
    #[prost(bytes="vec", tag="1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag="2")]
    pub eom: bool,
}
/// A Stream represents an instance of a gRPC stream between
/// a client and a server.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Stream {
    #[prost(uint64, tag="1")]
    pub id: u64,
}
/// A Request is a frame coming from a client. It is always
/// associated with a stream where the client assigns the stream
/// identifier. Servers will drop frames where the stream identifier
/// has no association (if a non-header frames are sent).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Request {
    #[prost(message, optional, tag="1")]
    pub stream: ::core::option::Option<Stream>,
    #[prost(oneof="request::Type", tags="2, 3, 4")]
    pub r#type: ::core::option::Option<request::Type>,
}
/// Nested message and enum types in `Request`.
pub mod request {
    #[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Type {
        #[prost(message, tag="2")]
        Headers(super::RequestHeaders),
        #[prost(message, tag="3")]
        Message(super::RequestMessage),
        #[prost(bool, tag="4")]
        RstStream(bool),
    }
}
/// RequestHeaders describe the unary or streaming call to make.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestHeaders {
    #[prost(string, tag="1")]
    pub method: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub metadata: ::core::option::Option<Metadata>,
    #[prost(message, optional, tag="3")]
    pub timeout: ::core::option::Option<::prost_types::Duration>,
}
/// A RequestMessage contains individual gRPC messages and a potential
/// end-of-stream (EOS) marker.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestMessage {
    #[prost(bool, tag="1")]
    pub has_message: bool,
    #[prost(message, optional, tag="2")]
    pub packet_message: ::core::option::Option<PacketMessage>,
    #[prost(bool, tag="3")]
    pub eos: bool,
}
/// A Response is a frame coming from a server. It is always
/// associated with a stream where the client assigns the stream
/// identifier. Clients will drop frames where the stream identifier
/// has no association.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Response {
    #[prost(message, optional, tag="1")]
    pub stream: ::core::option::Option<Stream>,
    #[prost(oneof="response::Type", tags="2, 3, 4")]
    pub r#type: ::core::option::Option<response::Type>,
}
/// Nested message and enum types in `Response`.
pub mod response {
    #[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Type {
        #[prost(message, tag="2")]
        Headers(super::ResponseHeaders),
        #[prost(message, tag="3")]
        Message(super::ResponseMessage),
        #[prost(message, tag="4")]
        Trailers(super::ResponseTrailers),
    }
}
/// ResponseHeaders contain custom metadata that are sent to the client
/// before any message or trailers (unless only trailers are sent).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseHeaders {
    #[prost(message, optional, tag="1")]
    pub metadata: ::core::option::Option<Metadata>,
}
/// ResponseMessage contains the data of a response to a call.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseMessage {
    #[prost(message, optional, tag="1")]
    pub packet_message: ::core::option::Option<PacketMessage>,
}
/// ResponseTrailers contain the status of a response and any custom metadata.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseTrailers {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::super::super::super::google::rpc::Status>,
    #[prost(message, optional, tag="2")]
    pub metadata: ::core::option::Option<Metadata>,
}
/// Strings are a series of values.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Strings {
    #[prost(string, repeated, tag="1")]
    pub values: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Metadata is for custom key values provided by a client or server
/// during a stream.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Metadata {
    #[prost(map="string, message", tag="1")]
    pub md: ::std::collections::HashMap<::prost::alloc::string::String, Strings>,
}
/// ICECandidate represents an ICE candidate.
/// From <https://github.com/pion/webrtc/blob/5f6baf73255598a7b4a7c9400bb0381acc9aa3dc/icecandidateinit.go>
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IceCandidate {
    #[prost(string, tag="1")]
    pub candidate: ::prost::alloc::string::String,
    #[prost(string, optional, tag="2")]
    pub sdp_mid: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(uint32, optional, tag="3")]
    pub sdpm_line_index: ::core::option::Option<u32>,
    #[prost(string, optional, tag="4")]
    pub username_fragment: ::core::option::Option<::prost::alloc::string::String>,
}
/// CallRequest is the SDP offer that the controlling side is making.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CallRequest {
    #[prost(string, tag="1")]
    pub sdp: ::prost::alloc::string::String,
    /// when disable_trickle is true, the init stage will be the only stage
    /// to be received in the response and the caller can expect the SDP
    /// to contain all ICE candidates.
    #[prost(bool, tag="2")]
    pub disable_trickle: bool,
}
/// CallResponseInitStage is the first and a one time stage that represents
/// the initial response to starting a call.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CallResponseInitStage {
    #[prost(string, tag="1")]
    pub sdp: ::prost::alloc::string::String,
}
/// CallResponseUpdateStage is multiply used to trickle in ICE candidates from
/// the controlled (answering) side.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CallResponseUpdateStage {
    #[prost(message, optional, tag="1")]
    pub candidate: ::core::option::Option<IceCandidate>,
}
/// CallResponse is the SDP answer that the controlled side responds with.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CallResponse {
    #[prost(string, tag="1")]
    pub uuid: ::prost::alloc::string::String,
    #[prost(oneof="call_response::Stage", tags="2, 3")]
    pub stage: ::core::option::Option<call_response::Stage>,
}
/// Nested message and enum types in `CallResponse`.
pub mod call_response {
    #[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Stage {
        #[prost(message, tag="2")]
        Init(super::CallResponseInitStage),
        #[prost(message, tag="3")]
        Update(super::CallResponseUpdateStage),
    }
}
/// CallUpdateRequest updates the call with additional info to the controlled side.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CallUpdateRequest {
    #[prost(string, tag="1")]
    pub uuid: ::prost::alloc::string::String,
    #[prost(oneof="call_update_request::Update", tags="2, 3, 4")]
    pub update: ::core::option::Option<call_update_request::Update>,
}
/// Nested message and enum types in `CallUpdateRequest`.
pub mod call_update_request {
    #[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Update {
        #[prost(message, tag="2")]
        Candidate(super::IceCandidate),
        #[prost(bool, tag="3")]
        Done(bool),
        #[prost(message, tag="4")]
        Error(super::super::super::super::super::google::rpc::Status),
    }
}
/// CallUpdateResponse contains nothing in response to a call update.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CallUpdateResponse {
}
/// ICEServer describes an ICE server.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IceServer {
    #[prost(string, repeated, tag="1")]
    pub urls: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, tag="2")]
    pub username: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub credential: ::prost::alloc::string::String,
}
/// WebRTCConfig represents parts of a WebRTC config.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WebRtcConfig {
    #[prost(message, repeated, tag="1")]
    pub additional_ice_servers: ::prost::alloc::vec::Vec<IceServer>,
    /// disable_trickle indicates if Trickle ICE should be used. Currently, both
    /// sides must both respect this setting.
    #[prost(bool, tag="2")]
    pub disable_trickle: bool,
}
/// AnswerRequestInitStage is the first and a one time stage that represents the
/// callers initial SDP request to the controlled (answerer) side.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AnswerRequestInitStage {
    #[prost(string, tag="1")]
    pub sdp: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub optional_config: ::core::option::Option<WebRtcConfig>,
    #[prost(message, optional, tag="3")]
    pub deadline: ::core::option::Option<::prost_types::Timestamp>,
}
/// AnswerRequestUpdateStage is multiply used to trickle in ICE candidates to
/// the controlled (answerer) side.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AnswerRequestUpdateStage {
    #[prost(message, optional, tag="1")]
    pub candidate: ::core::option::Option<IceCandidate>,
}
/// AnswerRequestDoneStage indicates the controller is done responding with candidates.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AnswerRequestDoneStage {
}
/// AnswerRequestErrorStage indicates the exchange has failed with an error.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AnswerRequestErrorStage {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::super::super::super::google::rpc::Status>,
}
/// AnswerRequestHeartbeatStage is sent periodically to verify liveness of answerer.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AnswerRequestHeartbeatStage {
}
/// AnswerRequest is the SDP offer that the controlling side is making via the answering
/// stream.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AnswerRequest {
    #[prost(string, tag="1")]
    pub uuid: ::prost::alloc::string::String,
    #[prost(oneof="answer_request::Stage", tags="2, 3, 4, 5, 6")]
    pub stage: ::core::option::Option<answer_request::Stage>,
}
/// Nested message and enum types in `AnswerRequest`.
pub mod answer_request {
    #[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Stage {
        #[prost(message, tag="2")]
        Init(super::AnswerRequestInitStage),
        #[prost(message, tag="3")]
        Update(super::AnswerRequestUpdateStage),
        /// done is sent when the requester is done sending information
        #[prost(message, tag="4")]
        Done(super::AnswerRequestDoneStage),
        /// error is sent any time before done
        #[prost(message, tag="5")]
        Error(super::AnswerRequestErrorStage),
        /// heartbeat is sent periodically to verify liveness of answerer
        #[prost(message, tag="6")]
        Heartbeat(super::AnswerRequestHeartbeatStage),
    }
}
/// AnswerResponseInitStage is the first and a one time stage that represents the
/// answerers initial SDP response to the controlling side.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AnswerResponseInitStage {
    #[prost(string, tag="1")]
    pub sdp: ::prost::alloc::string::String,
}
/// AnswerResponseUpdateStage is multiply used to trickle in ICE candidates to
/// the controlling side.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AnswerResponseUpdateStage {
    #[prost(message, optional, tag="1")]
    pub candidate: ::core::option::Option<IceCandidate>,
}
/// AnswerResponseDoneStage indicates the answerer is done responding with candidates.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AnswerResponseDoneStage {
}
/// AnswerResponseErrorStage indicates the exchange has failed with an error.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AnswerResponseErrorStage {
    #[prost(message, optional, tag="1")]
    pub status: ::core::option::Option<super::super::super::super::google::rpc::Status>,
}
/// AnswerResponse is the SDP answer that an answerer responds with.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AnswerResponse {
    #[prost(string, tag="1")]
    pub uuid: ::prost::alloc::string::String,
    #[prost(oneof="answer_response::Stage", tags="2, 3, 4, 5")]
    pub stage: ::core::option::Option<answer_response::Stage>,
}
/// Nested message and enum types in `AnswerResponse`.
pub mod answer_response {
    #[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Stage {
        #[prost(message, tag="2")]
        Init(super::AnswerResponseInitStage),
        #[prost(message, tag="3")]
        Update(super::AnswerResponseUpdateStage),
        /// done is sent when the answerer is done sending information
        #[prost(message, tag="4")]
        Done(super::AnswerResponseDoneStage),
        /// error is sent any time before done
        #[prost(message, tag="5")]
        Error(super::AnswerResponseErrorStage),
    }
}
/// OptionalWebRTCConfigRequest is the request for getting an optional WebRTC config
/// to use for the peer connection.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OptionalWebRtcConfigRequest {
}
/// OptionalWebRTCConfigResponse contains the optional WebRTC config
/// to use for the peer connection.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OptionalWebRtcConfigResponse {
    #[prost(message, optional, tag="1")]
    pub config: ::core::option::Option<WebRtcConfig>,
}
/// ConnectionCandidate describes the selected ICE candidate for one side of a WebRTC connection.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectionCandidate {
    #[prost(enumeration="IceCandidateType", tag="1")]
    pub r#type: i32,
    /// relay_address is the relay server address of this candidate; set only when type is
    /// RELAY, so the signaling server can classify the provider by matching against known
    /// coturn addresses.
    #[prost(string, tag="2")]
    pub relay_address: ::prost::alloc::string::String,
}
/// ReportConnectionMetadataRequest reports metadata about a WebRTC dial, per side: local is the
/// dialing SDK and remote is the robot.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportConnectionMetadataRequest {
    #[prost(message, optional, tag="1")]
    pub local: ::core::option::Option<ConnectionCandidate>,
    #[prost(message, optional, tag="2")]
    pub remote: ::core::option::Option<ConnectionCandidate>,
    #[prost(enumeration="SdkType", tag="3")]
    pub sdk_type: i32,
    /// reached_stage is the furthest dial checkpoint reached. READY indicates success; any earlier
    /// value is where a failed dial stopped.
    #[prost(enumeration="DialStage", tag="4")]
    pub reached_stage: i32,
    /// duration_ms is the wall-clock time from dial start to connection ready or to the failure.
    #[prost(uint32, tag="5")]
    pub duration_ms: u32,
    /// signaling_path is how the dial was signaled (cloud / local / mDNS); reported regardless of outcome.
    #[prost(enumeration="ConnectionSignalingPath", tag="6")]
    pub signaling_path: i32,
    /// failure_code is the gRPC status code of a failed dial
    #[prost(int32, tag="7")]
    pub failure_code: i32,
    /// sdk_version is the version of the dialing SDK (e.g. a semver or git tag).
    #[prost(string, tag="8")]
    pub sdk_version: ::prost::alloc::string::String,
}
/// ReportConnectionMetadataResponse is empty.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReportConnectionMetadataResponse {
}
/// ICECandidateType represents the type of ICE candidate selected for a WebRTC connection.
/// The signaling server further classifies RELAY by relay server specific provider from the address.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum IceCandidateType {
    Unspecified = 0,
    /// ICE_CANDIDATE_TYPE_HOST indicates a direct connection was established.
    Host = 1,
    /// ICE_CANDIDATE_TYPE_STUN indicates a STUN-assisted connection was established.
    Stun = 2,
    /// ICE_CANDIDATE_TYPE_RELAY indicates a TURN relay candidate was selected.
    Relay = 3,
}
impl IceCandidateType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            IceCandidateType::Unspecified => "ICE_CANDIDATE_TYPE_UNSPECIFIED",
            IceCandidateType::Host => "ICE_CANDIDATE_TYPE_HOST",
            IceCandidateType::Stun => "ICE_CANDIDATE_TYPE_STUN",
            IceCandidateType::Relay => "ICE_CANDIDATE_TYPE_RELAY",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "ICE_CANDIDATE_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "ICE_CANDIDATE_TYPE_HOST" => Some(Self::Host),
            "ICE_CANDIDATE_TYPE_STUN" => Some(Self::Stun),
            "ICE_CANDIDATE_TYPE_RELAY" => Some(Self::Relay),
            _ => None,
        }
    }
}
/// SDKType represents the Viam SDK used to establish the connection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SdkType {
    Unspecified = 0,
    Go = 1,
    Typescript = 2,
    PythonCpp = 3,
}
impl SdkType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SdkType::Unspecified => "SDK_TYPE_UNSPECIFIED",
            SdkType::Go => "SDK_TYPE_GO",
            SdkType::Typescript => "SDK_TYPE_TYPESCRIPT",
            SdkType::PythonCpp => "SDK_TYPE_PYTHON_CPP",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SDK_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
            "SDK_TYPE_GO" => Some(Self::Go),
            "SDK_TYPE_TYPESCRIPT" => Some(Self::Typescript),
            "SDK_TYPE_PYTHON_CPP" => Some(Self::PythonCpp),
            _ => None,
        }
    }
}
/// DialStage is the furthest checkpoint a WebRTC dial reached. READY means the dial succeeded; any
/// earlier value is the stage at which a failed dial stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DialStage {
    Unspecified = 0,
    /// DIAL_STAGE_SIGNALING_CONNECTED: the signaling channel was established.
    SignalingConnected = 1,
    /// DIAL_STAGE_CONFIG_FETCHED: ICE/TURN configuration was fetched from the signaling server.
    ConfigFetched = 2,
    /// DIAL_STAGE_OFFER_SENT: the SDP offer was sent to the signaling server (the Call was accepted).
    OfferSent = 3,
    /// DIAL_STAGE_ANSWER_RECEIVED: the answerer's SDP answer was received and applied.
    AnswerReceived = 4,
    /// DIAL_STAGE_ICE_CONNECTED: ICE connectivity was established (a candidate pair connected).
    IceConnected = 5,
    /// DIAL_STAGE_DTLS_CONNECTED: the DTLS handshake completed (peer connection connected) but the
    /// data channel is not yet open.
    DtlsConnected = 6,
    /// DIAL_STAGE_READY: the connection is fully ready (data channel open). This is success.
    Ready = 7,
}
impl DialStage {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            DialStage::Unspecified => "DIAL_STAGE_UNSPECIFIED",
            DialStage::SignalingConnected => "DIAL_STAGE_SIGNALING_CONNECTED",
            DialStage::ConfigFetched => "DIAL_STAGE_CONFIG_FETCHED",
            DialStage::OfferSent => "DIAL_STAGE_OFFER_SENT",
            DialStage::AnswerReceived => "DIAL_STAGE_ANSWER_RECEIVED",
            DialStage::IceConnected => "DIAL_STAGE_ICE_CONNECTED",
            DialStage::DtlsConnected => "DIAL_STAGE_DTLS_CONNECTED",
            DialStage::Ready => "DIAL_STAGE_READY",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "DIAL_STAGE_UNSPECIFIED" => Some(Self::Unspecified),
            "DIAL_STAGE_SIGNALING_CONNECTED" => Some(Self::SignalingConnected),
            "DIAL_STAGE_CONFIG_FETCHED" => Some(Self::ConfigFetched),
            "DIAL_STAGE_OFFER_SENT" => Some(Self::OfferSent),
            "DIAL_STAGE_ANSWER_RECEIVED" => Some(Self::AnswerReceived),
            "DIAL_STAGE_ICE_CONNECTED" => Some(Self::IceConnected),
            "DIAL_STAGE_DTLS_CONNECTED" => Some(Self::DtlsConnected),
            "DIAL_STAGE_READY" => Some(Self::Ready),
            _ => None,
        }
    }
}
/// ConnectionSignalingPath is how a WebRTC dial was signaled, derived from the signaling address.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ConnectionSignalingPath {
    Unspecified = 0,
    /// CONNECTION_SIGNALING_PATH_CLOUD_SIGNALED: signaled through app's signaling server.
    CloudSignaled = 1,
    /// CONNECTION_SIGNALING_PATH_MDNS_LOCAL: signaled over an mDNS-discovered local-network path.
    MdnsLocal = 2,
    /// CONNECTION_SIGNALING_PATH_LOCAL: signaled through a loopback/private-address signaling server
    /// (e.g. a machine's own signaling server) without mDNS discovery.
    Local = 3,
}
impl ConnectionSignalingPath {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ConnectionSignalingPath::Unspecified => "CONNECTION_SIGNALING_PATH_UNSPECIFIED",
            ConnectionSignalingPath::CloudSignaled => "CONNECTION_SIGNALING_PATH_CLOUD_SIGNALED",
            ConnectionSignalingPath::MdnsLocal => "CONNECTION_SIGNALING_PATH_MDNS_LOCAL",
            ConnectionSignalingPath::Local => "CONNECTION_SIGNALING_PATH_LOCAL",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "CONNECTION_SIGNALING_PATH_UNSPECIFIED" => Some(Self::Unspecified),
            "CONNECTION_SIGNALING_PATH_CLOUD_SIGNALED" => Some(Self::CloudSignaled),
            "CONNECTION_SIGNALING_PATH_MDNS_LOCAL" => Some(Self::MdnsLocal),
            "CONNECTION_SIGNALING_PATH_LOCAL" => Some(Self::Local),
            _ => None,
        }
    }
}
// @@protoc_insertion_point(module)
