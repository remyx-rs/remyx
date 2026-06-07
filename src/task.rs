use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{
    Stream,
    future::{self},
    stream::{FusedStream, FuturesUnordered},
};

use crate::runtime::{self, JoinError, JoinHandleFutOf};

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

pub struct Pending<Runtime, Message>
where
    Runtime: runtime::Runtime,
    Message: Send,
{
    queue: FuturesUnordered<JoinHandleFutOf<Runtime, Message>>,
}

impl<Runtime, Message> Default for Pending<Runtime, Message>
where
    Runtime: runtime::Runtime,
    Message: Send,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Runtime, Message> Pending<Runtime, Message>
where
    Runtime: runtime::Runtime,
    Message: Send,
{
    pub fn new() -> Self {
        Self {
            queue: FuturesUnordered::new(),
        }
    }

    pub fn register(&self, handle: JoinHandleFutOf<Runtime, Message>) {
        self.queue.push(handle);
    }
}

impl<Runtime, Message> Stream for Pending<Runtime, Message>
where
    Runtime: runtime::Runtime,
    Message: Send,
{
    type Item = Result<Message, JoinError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        Pin::new(&mut this.queue).poll_next(cx)
    }
}

impl<Runtime, Message> FusedStream for Pending<Runtime, Message>
where
    Runtime: runtime::Runtime,
    Message: Send,
{
    fn is_terminated(&self) -> bool {
        self.queue.is_terminated()
    }
}
