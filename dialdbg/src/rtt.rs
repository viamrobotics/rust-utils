use anyhow::Result;
use viam::rpc::dial::ViamChannel;

// Returns the average round-trip-time over num_pings for the passed-in channel.
pub(crate) async fn measure_rtt(_ch: ViamChannel, _num_pings: usize) -> Result<f64> {
    Ok(0.0)
}
