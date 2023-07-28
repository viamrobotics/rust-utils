/// Tests dialdbg against an echo server running on localhost:$SERVER_PORT.
use crate::{main_inner, Args};
use std::env;

#[tokio::test]
async fn dial() {
    let mut args = Args::default();
    let port = env::var("SERVER_PORT").unwrap().to_owned();
    args.uri = Some(["localhost:".to_string(), port].join(""));

    // NOTE(benjirewis): simply assert that main_inner returned no error. It may be overkill right
    // now to assert anything about the output.
    assert!(main_inner(args).await.is_ok());
}
