use std::ops::Add;
use std::time::{Duration, Instant};

use remyx::runtime::Runtime;
use remyx::runtime::time::{Time, TimedOut};

pub async fn sleep_delays_for_at_least_requested_duration<R: Runtime>() {
    let now = Instant::now();
    let duration = Duration::from_millis(250);
    R::Time::sleep(duration).await;

    assert!(Instant::now() >= now.add(duration));
}

pub async fn timeout_returns_ok_when_future_completes_in_time<R: Runtime>() {
    let result = R::Time::timeout(Duration::from_millis(250), async move { "hello world!" }).await;
    assert!(matches!(result, Ok("hello world!")));
}

pub async fn timeout_returns_timed_out_when_future_exceeds_limit<R: Runtime>() {
    let result = R::Time::timeout(Duration::from_millis(125), async move {
        R::Time::sleep(Duration::from_millis(250)).await;
        "hello world!"
    })
    .await;
    assert!(matches!(result, Err(TimedOut {})));
}

pub async fn timeout_with_zero_duration_returns_immediate_future<R: Runtime>() {
    let result = R::Time::timeout(Duration::ZERO, async move { "hello world!" }).await;
    assert!(matches!(result, Ok("hello world!")));
}
