use futures::future::LocalBoxFuture;
use futures::{StreamExt, stream::LocalBoxStream};
use std::hash::{DefaultHasher, Hasher};
#[cfg(feature = "crossterm")]
use std::io;
use std::sync::atomic::AtomicU64;
use std::{hash::Hash, pin::Pin, task::Poll};

pub struct Stream<Message> {
    sources: Vec<Source<Message>>,
}

impl<Message> Stream<Message> {
    pub fn init(sources: Vec<Source<Message>>) -> Self {
        Self { sources }
    }

    pub fn add(&mut self, source: Source<Message>) {
        self.sources.push(source)
    }

    pub fn diff(&mut self, sources: Vec<Source<Message>>) {
        self.sources
            .retain(|current| sources.iter().any(|incoming| incoming.id == current.id));

        for source in sources {
            if self.sources.iter().all(|current| current.id != source.id) {
                self.sources.push(source);
            }
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
    pub const HASH_MASK: u64 = 1111_1111_1111_1110;
    const AUTO_INCREMENTAL_MASK: u64 = 0000_0000_0000_0001;

    pub fn new<O: 'static>(f: fn() -> O) -> Self
    where
        O: futures::Stream<Item = Message>,
    {
        let id: u64 = ((f as usize as u64) << 1) & Self::HASH_MASK;
        let stream = futures::stream::once(async move { f() }).flatten();
        Self {
            id,
            stream: Box::pin(stream),
        }
    }
    pub fn with<I: 'static, O: 'static>(data: I, f: fn(&I) -> O) -> Self
    where
        I: Hash,
        O: futures::Stream<Item = Message>,
    {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);

        let id: u64 = ((f as usize as u64 & hasher.finish()) << 1) & Self::HASH_MASK;
        let stream = futures::stream::once(async move { f(&data) }).flatten();

        Self {
            id,
            stream: Box::pin(stream),
        }
    }

    pub(crate) fn future(fut: LocalBoxFuture<'static, Message>) -> Self {
        static ID: AtomicU64 = AtomicU64::new(0);

        let id = (ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst) << 1)
            | Self::AUTO_INCREMENTAL_MASK;

        Self {
            id,
            stream: Box::pin(futures::stream::once(fut)),
        }
    }
}

#[cfg(feature = "crossterm")]
pub fn terminal_event() -> impl futures::Stream<Item = io::Result<crossterm::event::Event>> {
    crossterm::event::EventStream::new()
}
