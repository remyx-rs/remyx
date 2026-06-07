use crate::runtime::time::Time;

pub struct TokioTime {}

impl Time for TokioTime {
    fn sleep(duration: std::time::Duration) -> impl Future<Output = ()> {
        tokio::time::sleep(duration)
    }
}
