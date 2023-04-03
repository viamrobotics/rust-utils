/// Tests unary, server, and bidi streaming with simple echo requests. To run, simply
/// update the credentials and uri as necessary.
use anyhow::Result;
use std::env;
use std::sync::{Arc, RwLock};
use viam_rust_utils::gen::proto::rpc::examples::echo::v1::echo_service_client::EchoServiceClient;
use viam_rust_utils::gen::proto::rpc::examples::echo::v1::{
    EchoBiDiRequest, EchoMultipleRequest, EchoRequest,
};
use viam_rust_utils::rpc::dial;

async fn dial_direct() -> Result<dial::ViamChannel> {
    let port = env::var("SERVER_PORT").unwrap().to_owned();
    let uri = ["localhost:".to_string(), port].join("");

    dial::DialOptions::builder()
        .uri(&uri)
        .without_credentials()
        .insecure()
        .disable_webrtc()
        .connect()
        .await
}

#[tokio::test]
async fn test_dial_direct_unary() -> Result<()> {
    let c = dial_direct().await?;

    let mut service = EchoServiceClient::new(c);
    let echo_request = EchoRequest {
        message: "hi".to_string(),
    };
    let resp = service.echo(echo_request).await?.into_inner();
    assert_eq!(resp.message, "hi".to_string());

    Ok(())
}

#[tokio::test]
async fn test_dial_direct_server_stream() -> Result<()> {
    let c = dial_direct().await?;

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
async fn test_dial_direct_bidi() -> Result<()> {
    let c = dial_direct().await?;

    let received = Arc::new(RwLock::new(0));
    let received_async = Arc::clone(&received);

    let bidi_stream = async_stream::stream! {
        for i in 0..3 {
            loop {
                // We need to wait a small amount of time between each request/response count
                // check, otherwise we lock up the main thread.
                let sleep_time = std::time::Duration::from_millis(10);
                tokio::time::sleep(sleep_time).await;

                // Wait until we have received one response for each request before sending the
                // next request. This allows requests/response to be interleaved.
                let value = received_async.read().unwrap();
                if *value == i {
                    break;
                }
            }

            let request =
            EchoBiDiRequest {
                message: i.to_string()
            };
            yield request;
        }
    };

    let mut service = EchoServiceClient::new(c);
    let mut bidi_resp = service.echo_bi_di(bidi_stream).await?.into_inner();

    for i in 0..3 {
        let resp = bidi_resp.message().await?.unwrap();
        assert_eq!(resp.message, i.to_string());

        let mut count = received.write().unwrap();
        *count += 1;
        drop(count);
    }

    Ok(())
}

async fn dial_webrtc() -> Result<dial::ViamChannel> {
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
async fn test_dial_webrtc_unary() -> Result<()> {
    let c = dial_webrtc().await?;

    let mut service = EchoServiceClient::new(c);
    let echo_request = EchoRequest {
        message: "hi".to_string(),
    };
    let resp = service.echo(echo_request).await?.into_inner();
    assert_eq!(resp.message, "hi".to_string());

    Ok(())
}

#[tokio::test]
async fn test_dial_webrtc_server_stream() -> Result<()> {
    let c = dial_webrtc().await?;

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
async fn test_dial_webrtc_bidi() -> Result<()> {
    let c = dial_webrtc().await?;

    // TODO(RSDK-2414): ideally we should mix the timing of our requests and responses truly ensure that we
    // support bi-directionality.
    let bidi_stream = async_stream::stream! {
        for i in 0..3 {
            let request =
            EchoBiDiRequest {
                message: i.to_string()
            };
            yield request;
        }
    };

    let mut service = EchoServiceClient::new(c);
    let mut bidi_resp = service.echo_bi_di(bidi_stream).await?.into_inner();

    for i in 0..3 {
        let resp = bidi_resp.message().await?.unwrap();
        assert_eq!(resp.message, i.to_string());
    }

    Ok(())
}
