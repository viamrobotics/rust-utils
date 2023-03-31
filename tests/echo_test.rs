/// Tests unary, server, and bidi streaming with simple echo requests. To run, simply
/// update the credentials and uri as necessary.
use anyhow::Result;
use std::env;
use std::sync::{Arc, Mutex};
use viam_rust_utils::gen::proto::rpc::examples::echo::v1::echo_service_client::EchoServiceClient;
use viam_rust_utils::gen::proto::rpc::examples::echo::v1::{
    EchoBiDiRequest, EchoMultipleRequest, EchoRequest,
};
use viam_rust_utils::rpc::dial;

async fn dial() -> Result<dial::ViamChannel> {
    let port = env::var("SERVER_PORT").unwrap().to_owned();
    let uri = ["localhost:".to_string(), port].join("");

    dial::DialOptions::builder()
        .uri(&uri)
        .without_credentials()
        .insecure()
        .connect()
        .await
}

#[tokio::test]
async fn test_webrtc_unary() -> Result<()> {
    let c = dial().await?;

    let mut service = EchoServiceClient::new(c);
    let echo_request = EchoRequest {
        message: "hi".to_string(),
    };
    let resp = service.echo(echo_request).await?.into_inner();
    assert_eq!(resp.message, "hi".to_string());

    Ok(())
}

#[tokio::test]
async fn test_webrtc_server_stream() -> Result<()> {
    let c = dial().await?;

    let mut service = EchoServiceClient::new(c);
    let multi_echo_request = EchoMultipleRequest {
        message: "hello?".to_string(),
    };

    let mut expected = vec!["h", "e", "l", "l", "o", "?"];
    expected.reverse();

    let mut resp = service
        .echo_multiple(multi_echo_request)
        .await?
        .into_inner();
    while let Some(resp) = resp.message().await? {
        assert_eq!(resp.message, expected.pop().unwrap().to_string())
    }
    assert!(expected.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_webrtc_bidi() -> Result<()> {
    env_logger::init();
    let c = dial().await?;

    // This variable gates when we send the last request.
    let send_last = Arc::new(Mutex::new(false));
    let send_last_async = Arc::clone(&send_last);

    let bidi_stream = async_stream::stream! {
        for i in 0..2 {
            let request =
            EchoBiDiRequest {
                message: i.to_string()
            };
            yield request;
        }

        log::info!("waiting...");
        loop {
            let sleep_time = std::time::Duration::from_millis(1000);
            tokio::time::sleep(sleep_time).await;

            let lock = send_last_async.lock().unwrap();
            if *lock {
                drop(lock);
                break;
            }
            drop(lock);
            log::info!("still waiting...");
        }
        // let sleep_time = std::time::Duration::from_millis(1000);
        // tokio::time::sleep(sleep_time).await;
        log::info!("finished waiting!");

        yield EchoBiDiRequest { message: 2.to_string() };
    };

    let mut service = EchoServiceClient::new(c);
    log::info!("making the call...");
    let mut bidi_resp = service.echo_bi_di(bidi_stream).await?.into_inner();
    log::info!("made the call!");

    let resp = bidi_resp.message().await?.unwrap();
    assert_eq!(resp.message, "0");
    log::info!("got 0!");

    let mut lock = send_last.lock().unwrap();
    *lock = true;
    drop(lock);

    let resp = bidi_resp.message().await?.unwrap();
    assert_eq!(resp.message, "1");
    log::info!("got 1!");

    let resp = bidi_resp.message().await?.unwrap();
    assert_eq!(resp.message, "2");
    log::info!("got 2!");

    Ok(())
}
