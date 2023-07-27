use anyhow::{anyhow, Result};
use std::{ops::Add, time};
use viam_rust_utils::gen::proto::rpc::examples::echo::v1::{
    echo_service_client::EchoServiceClient, EchoRequest,
};
use viam_rust_utils::rpc::dial::ViamChannel;

// Returns the average round-trip-time over num_pings for the passed-in channel.
pub(crate) async fn measure_rtt(ch: ViamChannel, num_pings: u32) -> Result<time::Duration> {
    let mut total_ping = time::Duration::new(0, 0);
    for _ in 0..num_pings {
        let start = time::Instant::now();

        // Send an echo request across the channel. It's unlikely the remote will be able to
        // respond to this request, but we'll still get a good sense of RTT.
        let mut service = EchoServiceClient::new(ch.clone());
        let echo_request = EchoRequest {
            message: "dialdbg".to_string(),
        };
        service.echo(echo_request).await.ok();

        total_ping = total_ping.add(time::Instant::now().duration_since(start));
    }
    if let Some(avg_ping) = total_ping.checked_div(num_pings) {
        return Ok(avg_ping);
    }
    Err(anyhow!("cannot divide by zero"))
}
