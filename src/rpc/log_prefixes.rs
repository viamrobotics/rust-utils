// log_prefixes.rs contains prefixes for rpc log messages. The associated dialdbg tool (another
// workspace in this crate), parses these prefixes to determine dial behavior. Ensure modifications
// to this file are accordingly respected in dialdbg.

pub const MDNS_ADDRESS_FOUND: &'static str = "Found address via mDNS";
pub const MDNS_QUERY_ATTEMPT: &'static str = "Attempting to connect via mDNS";
pub const MDNS_QUERY_SUCCESS: &'static str = "Connected via mDNS";

pub const ACQUIRING_AUTH_TOKEN: &'static str = "Acquiring auth token";
pub const ACQUIRED_AUTH_TOKEN: &'static str = "Acquired auth token";

pub const DIAL_ATTEMPT: &'static str = "Dialing";
pub const DIALED_GRPC: &'static str = "Connected via gRPC";
pub const DIALED_WEBRTC: &'static str = "Connected via WebRTC";

pub const CANDIDATE_SELECTED: &'static str = "Selected candidate pair";
