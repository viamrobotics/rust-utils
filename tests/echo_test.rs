use anyhow::Result;
use viam_rust_utils::gen::proto::rpc::examples::echo::v1::echo_service_client::EchoServiceClient;
use viam_rust_utils::gen::proto::rpc::examples::echo::v1::{
    EchoBiDiRequest, EchoMultipleRequest, EchoRequest,
};
use viam_rust_utils::rpc::dial;

#[tokio::test]
/// Tests unary, server, and bidi streaming with simple echo requests. To run, simply
/// update the credentials and uri as necessary.
async fn test_echo() -> Result<()> {
    let c = dial::DialOptions::builder()
        .uri("localhost:8080")
        .without_credentials()
        .insecure()
        .allow_downgrade()
        .connect()
        .await?;

    // Unary case

    let mut service = EchoServiceClient::new(c);
    let echo_request = EchoRequest {
        message: "hi".to_string(),
    };
    let resp = service.echo(echo_request).await?.into_inner();
    assert_eq!(resp.message, "hi".to_string());

    let multi_echo_request = EchoMultipleRequest {
        message: "hello?".to_string(),
    };

    // Server-stream case

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

    // // Bi-directional case
    //
    // let bidi_stream = async_stream::stream! {
    //     for i in 0..3 {
    //         let request =
    //         EchoBiDiRequest {
    //             message: i.to_string()
    //         };
    //         yield request;
    //     }
    // };
    //
    // let mut bidi_resp = service.echo_bi_di(bidi_stream).await?.into_inner();
    // while let Some(resp) = bidi_resp.message().await? {
    //     println!("Bidi response: {resp:?}");
    // }

    Ok(())
}
