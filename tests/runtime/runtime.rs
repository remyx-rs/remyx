use std::time::Duration;

use remyx::runtime::time::Time;
use remyx::runtime::{JoinHandle, Runtime, oneshot};

pub async fn block_on_returns_completed_future_value<R: Runtime>() {
    assert_eq!(async { "hello world!" }.await, "hello world!");
}

pub async fn defer_spawn_returns_joined_task_result<R: Runtime>() {
    let handle = R::defer(1, 1024, async move {
        R::Time::sleep(Duration::from_millis(250)).await;
    });
    let result = handle.spawn(async move { "hello world!" }).await.unwrap();
    assert!(matches!(result.into_future().await, Ok("hello world!")));
}

pub async fn defer_spawn_after_runtime_dropped_returns_disconnected<R: Runtime>() {
    let handle = R::defer(1, 1024, async move {});

    R::Time::sleep(Duration::from_millis(250)).await;
    let result = handle.spawn(async move { "hello world!" }).await;

    assert!(matches!(result, Err(oneshot::TryRecvError::Disconnected)));
}

pub async fn defer_join_returns_deferred_future_output<R: Runtime>() {
    let mut handle = R::defer(1, 1024, async move {
        R::Time::sleep(Duration::from_millis(250)).await;
        "hello world!"
    });
    assert!(matches!(handle.join().await, Ok("hello world!")));
}

pub async fn spawn_local_panics_on_multi_threaded_runtime<R: Runtime>() {
    R::spawn_local(async {});
}

pub async fn spawn_local_returns_result_on_single_threaded_runtime<R: Runtime>() {
    let result = R::spawn_local(async move { "hello world!" })
        .into_future()
        .await;
    assert!(matches!(result, Ok("hello world!")));
}
