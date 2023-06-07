// strings.rs contains substrings of dial log messages. These constants are admittedly the most
// fragile piece of dialdbg's code. If dialdbg is not behaving as expected, ensure these constants
// still match logs output by dial. Raw dial logs can be output by setting the DIALDBG_DEVELOPMENT
// environment variable.

pub(crate) const MDNS_ADDRESS_FOUND: &'static str = "Found address via mDNS";
pub(crate) const MDNS_QUERY_ATTEMPT: &'static str = "Attempting to connect via mDNS";
pub(crate) const MDNS_QUERY_SUCCESS: &'static str = "Connected via mDNS";

pub(crate) const ACQUIRING_AUTH_TOKEN: &'static str = "Acquiring auth token";
pub(crate) const ACQUIRED_AUTH_TOKEN: &'static str = "Acquired auth token";

pub(crate) const DIAL_ATTEMPT: &'static str = "Dialing";
pub(crate) const DIALED_GRPC: &'static str = "Connected via gRPC";
pub(crate) const DIALED_WEBRTC: &'static str = "Connected via WebRTC";

pub(crate) const CANDIDATE_SELECTED: &'static str = "Selected candidate pair";

// This prefix is prepended in dialdbg when connect returns an error. It is not
// from dial itself.
pub(crate) const DIAL_ERROR_PREFIX: &'static str = "unexpected dial connect error";
