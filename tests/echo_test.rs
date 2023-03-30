/// Tests unary, server, and bidi streaming with simple echo requests. To run, simply
/// update the credentials and uri as necessary.
use anyhow::Result;
use std::env;
use viam_rust_utils::gen::proto::rpc::examples::echo::v1::echo_service_client::EchoServiceClient;
use viam_rust_utils::gen::proto::rpc::examples::echo::v1::{
    EchoBiDiRequest, EchoMultipleRequest, EchoRequest,
};
use viam_rust_utils::rpc::dial;

async fn dial() -> Result<dial::ViamChannel, anyhow::Error> {
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
    let c = dial().await?;

    let bidi_stream = async_stream::stream! {
        for i in 0..3 {
            let request =
            EchoBiDiRequest {
                message: i.to_string()
            };
            yield request;
        }
    };

    let mut expected = vec!["0", "1", "?"];
    expected.reverse();

    let mut service = EchoServiceClient::new(c);
    let mut bidi_resp = service.echo_bi_di(bidi_stream).await?.into_inner();
    while let Some(resp) = bidi_resp.message().await? {
        assert_eq!(resp.message, expected.pop().unwrap().to_string())
    }
    assert!(expected.is_empty());

    Ok(())
}
