use futures::future::{self};

pub struct Task<Message> {
    fut: future::BoxFuture<'static, Message>,
}

impl<Message> Task<Message> {
    pub fn new<Fut>(fut: Fut) -> Self
    where
        Fut: Future<Output = Message> + Send + 'static,
    {
        Self { fut: Box::pin(fut) }
    }
}

impl<Message> Future for Task<Message> {
    type Output = Message;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        self.fut.as_mut().poll(cx)
    }
}
