use futures::{StreamExt, stream::LocalBoxStream};
use std::hash::{DefaultHasher, Hasher};
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
    pub fn new<S>(f: fn() -> S) -> Self
    where
        S: futures::Stream<Item = Message> + 'static,
    {
        let id: u64 = f as usize as u64;
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

        let id: u64 = f as usize as u64 & hasher.finish();
        let stream = futures::stream::once(async move { f(&data) }).flatten();
        Self {
            id,
            stream: Box::pin(stream),
        }
    }
}
