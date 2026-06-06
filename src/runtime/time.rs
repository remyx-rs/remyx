use std::{
    pin::pin,
    time::{Duration, Instant},
};

use futures::FutureExt;

pub trait Time {
    fn sleep(duration: Duration) -> impl Future<Output = ()> + Send;

    fn timeout_at<F: Future>(
        deadline: Instant,
        fut: F,
    ) -> impl Future<Output = Result<F::Output, TimedOut>> {
        let duration = deadline.saturating_duration_since(Instant::now());
        Self::timeout(duration, fut)
    }

    fn timeout<F: Future>(
        duration: Duration,
        fut: F,
    ) -> impl Future<Output = Result<F::Output, TimedOut>> {
        async move {
            let mut sleep = pin!(Self::sleep(duration).fuse());
            let mut fut = pin!(fut.fuse());
            futures::select_biased! {
                result = fut => {
                    Ok(result)
                }
                _ = sleep => {
                    Err(TimedOut {})
                }
            }
        }
    }
}

pub struct TimedOut {}
