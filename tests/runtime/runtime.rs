use remyx::runtime::{JoinHandle, Runtime};

#[allow(clippy::extra_unused_type_parameters)]
pub async fn block_on_returns_completed_future_value<R: Runtime>(_rt: &R) {
    assert_eq!(async { "hello world!" }.await, "hello world!");
}

pub async fn spawn_returns_result<R: Runtime>(rt: &R) {
    let result = rt.spawn(async move { "hello world!" }).into_future().await;
    assert!(matches!(result, Ok("hello world!")));
}
