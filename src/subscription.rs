use futures::Stream;
use futures::stream::{FusedStream, SelectAll};
use futures::{StreamExt, stream::LocalBoxStream};
use std::hash::Hash;
use std::hash::{DefaultHasher, Hasher};
use std::mem;
use std::pin::Pin;

pub struct Set<Message> {
    subscriptions: SelectAll<Subscription<Message>>,
}

impl<Message> Default for Set<Message> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Message> Set<Message> {
    pub fn new() -> Self {
        Self {
            subscriptions: SelectAll::new(),
        }
    }

    fn add(&mut self, subscription: Subscription<Message>) {
        let registered = self
            .subscriptions
            .iter()
            .any(|current| current.id == subscription.id);

        if !registered {
            self.subscriptions.push(subscription);
        }
    }

    pub fn diff(&mut self, subscriptions: Vec<Subscription<Message>>) {
        for subscription in mem::take(&mut self.subscriptions) {
            let was_previously = subscriptions
                .iter()
                .any(|incoming| incoming.id == subscription.id);

            if was_previously {
                self.subscriptions.push(subscription);
            }
        }

        for subscription in subscriptions {
            self.add(subscription);
        }
    }
}

impl<Message> Stream for Set<Message> {
    type Item = Message;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this.subscriptions.is_empty() {
            return std::task::Poll::Pending;
        }
        Pin::new(&mut this.subscriptions).poll_next(cx)
    }
}

impl<Message> FusedStream for Set<Message> {
    fn is_terminated(&self) -> bool {
        false
    }
}

pub struct Subscription<Message> {
    id: u64,
    stream: LocalBoxStream<'static, Message>,
}

impl<Message: 'static> Subscription<Message> {
    pub fn new<Stream>(f: fn() -> Stream) -> Self
    where
        Stream: futures::Stream<Item = Message> + 'static,
    {
        let id: u64 = f as usize as u64;
        let stream = futures::stream::once(async move { f() }).flatten();
        Self {
            id,
            stream: Box::pin(stream),
        }
    }

    pub fn with<I, Stream>(data: I, f: fn(&I) -> Stream) -> Self
    where
        I: Hash + 'static,
        Stream: futures::Stream<Item = Message> + 'static,
    {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);

        let id: u64 = f as usize as u64 & hasher.finish();
        let stream = futures::stream::once(async move { f(&data) }).flatten();
        Self {
            id,
            stream: Box::pin(stream),
        }
    }
}

impl<Message> Stream for Subscription<Message> {
    type Item = Message;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.stream.as_mut().poll_next(cx)
    }
}
