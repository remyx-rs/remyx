#[cfg(feature = "crossterm")]
use std::io;
use std::{hash::Hash, pin::Pin, task::Poll};

use futures::{StreamExt, stream::LocalBoxStream};

pub struct Stream<Message> {
    sources: Vec<Source<Message>>,
}

impl<Message> Stream<Message> {
    pub fn init(sources: Vec<Source<Message>>) -> Self {
        Self { sources }
    }

    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
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
    ) -> std::task::Poll<Option<Self::Item>> {
        for source in &mut self.sources {
            match source.stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(message)) => {
                    return Poll::Ready(Some(message));
                }
                Poll::Ready(None) => {
                    // Source ended; ignore for now
                }
                Poll::Pending => {}
            }
        }

        Poll::Pending
    }
}

pub struct Source<Message> {
    id: u64,
    stream: LocalBoxStream<'static, Message>,
}

impl<Message> Source<Message> {
    pub fn new<O: 'static>(f: fn() -> O) -> Self
    where
        O: futures::Stream<Item = Message>,
    {
        let id: u64 = f as usize as u64;
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
        // TODO: Compose this with part of fn pointer and other part with data hash
        let id: u64 = f as usize as u64;
        let stream = futures::stream::once(async move { f(&data) }).flatten();

        Self {
            id,
            stream: Box::pin(stream),
        }
    }
}

#[cfg(feature = "crossterm")]
pub fn terminal_event() -> impl futures::Stream<Item = io::Result<crossterm::event::Event>> {
    crossterm::event::EventStream::new()
}
