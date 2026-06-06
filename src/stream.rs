use futures::{StreamExt, stream::LocalBoxStream};
use std::hash::{DefaultHasher, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::{hash::Hash, pin::Pin, task::Poll};

pub type LocalBoxFusedStream<O> = Pin<Box<dyn futures::stream::FusedStream<Item = O>>>;

pub struct Stream<Message> {
    sources: Vec<Source<Message>>,
}

impl<Message> Stream<Message> {
    pub fn init(sources: Vec<Source<Message>>) -> Self {
        Self { sources }
    }

    pub fn add(&mut self, source: Source<Message>) {
        if self.sources.iter().all(|current| current.id != source.id) {
            self.sources.push(source)
        }
    }

    pub fn diff(&mut self, sources: Vec<Source<Message>>) {
        self.sources
            .retain(|current| sources.iter().any(|incoming| incoming.id == current.id));

        for source in sources {
            self.add(source);
        }
    }
}

impl<Message> futures::Stream for Stream<Message> {
    type Item = Message;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut index = 0;
        while index < self.sources.len() {
            match self.sources[index].stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(message)) => {
                    return Poll::Ready(Some(message));
                }
                Poll::Ready(None) => {
                    self.sources.remove(index);
                }
                Poll::Pending => {
                    index += 1;
                }
            }
        }
        Poll::Pending
    }
}

impl<Message> futures::stream::FusedStream for Stream<Message> {
    fn is_terminated(&self) -> bool {
        false
    }
}

pub struct Source<Message> {
    id: u64,
    stream: LocalBoxStream<'static, Message>,
}

impl<Message: 'static> Source<Message> {
    const STATIC_MASK: u64 = 1;
    const DYNAMIC_MASK: u64 = 0o1;

    pub fn new<S>(f: fn() -> S) -> Self
    where
        S: futures::Stream<Item = Message> + 'static,
    {
        let id: u64 = ((f as usize as u64) << 1) & Self::STATIC_MASK;
        let stream = futures::stream::once(async move { f() }).flatten();
        Self {
            id,
            stream: Box::pin(stream),
        }
    }

    pub fn with<I, S>(data: I, f: fn(&I) -> S) -> Self
    where
        I: Hash + 'static,
        S: futures::Stream<Item = Message> + 'static,
    {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);

        let id: u64 = ((f as usize as u64 & hasher.finish()) << 1) & Self::STATIC_MASK;
        let stream = futures::stream::once(async move { f(&data) }).flatten();

        Self {
            id,
            stream: Box::pin(stream),
        }
    }

    pub fn future<F>(fut: F) -> Self
    where
        F: Future<Output = Message> + 'static,
    {
        static ID: AtomicU64 = AtomicU64::new(0);

        let id = (ID.fetch_add(1, Ordering::SeqCst) << 1) | Self::DYNAMIC_MASK;
        Self {
            id,
            stream: Box::pin(futures::stream::once(fut)),
        }
    }
}
