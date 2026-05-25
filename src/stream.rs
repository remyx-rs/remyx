#[cfg(feature = "crossterm")]
use std::io;
use std::{pin::Pin, task::Poll};

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
    stream: Pin<Box<dyn futures::Stream<Item = Message>>>,
}

impl<Message> Source<Message> {
    pub fn init<O: 'static>(f: fn() -> O) -> Self
    where
        O: futures::Stream<Item = Message>,
    {
        let id: u64 = f as usize as u64;

        Self {
            id,
            stream: Box::pin(f()),
        }
    }
}

#[cfg(feature = "crossterm")]
pub fn terminal_event() -> impl futures::Stream<Item = io::Result<crossterm::event::Event>> {
    crossterm::event::EventStream::new()
}
