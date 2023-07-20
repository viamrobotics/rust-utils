use crate::{main_inner, Args};

#[tokio::test]
async fn dial() {
    let mut args = Args::default();
    args.uri = Some("[redacted]".to_string());
    args.credential = Some("[redacted]".to_string());

    // NOTE(benjirewis): simply assert that main_inner retunred no error. It may be overkill right
    // now to assert anything about the output.
    assert!(main_inner(args).await.is_ok());
}
