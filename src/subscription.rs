use crate::terminal;
use crossterm::event::{self, KeyEvent};
use futures::Stream;
use futures::stream::LocalBoxStream;
use futures::stream::StreamExt;
use futures::stream::{FusedStream, SelectAll};
use std::any::TypeId;
use std::hash::Hash;
use std::hash::{DefaultHasher, Hasher};
use std::pin::Pin;
use std::{future, mem};

pub struct Active<Message> {
    sources: SelectAll<Source<Message>>,
}

impl<Message: 'static> Default for Active<Message> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Message: 'static> Active<Message> {
    pub fn new() -> Self {
        Self {
            sources: SelectAll::new(),
        }
    }

    fn add<Terminal: terminal::Terminal>(
        &mut self,
        terminal: &mut Terminal,
        subscription: Subscription<Terminal, Message>,
    ) {
        let registered = self
            .sources
            .iter()
            .any(|current| current.id == subscription.id);

        if !registered {
            let source = subscription.build(terminal);
            self.sources.push(source);
        }
    }

    pub fn diff<Terminal: terminal::Terminal>(
        &mut self,
        terminal: &mut Terminal,
        subscriptions: Vec<Subscription<Terminal, Message>>,
    ) {
        for source in mem::take(&mut self.sources) {
            let still_present = subscriptions
                .iter()
                .any(|incoming| incoming.id == source.id);

            if still_present {
                self.sources.push(source);
            }
        }

        for subscription in subscriptions {
            self.add(terminal, subscription);
        }
    }
}

impl<Message> Stream for Active<Message> {
    type Item = Message;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this.sources.is_empty() {
            return std::task::Poll::Pending;
        }
        Pin::new(&mut this.sources).poll_next(cx)
    }
}

impl<Message> FusedStream for Active<Message> {
    fn is_terminated(&self) -> bool {
        false
    }
}

type StreamFn<Terminal, Message> =
    Box<dyn FnOnce(&mut Terminal) -> LocalBoxStream<'static, Message>>;

pub struct Subscription<Terminal, Message>
where
    Terminal: terminal::Terminal,
{
    id: u64,
    builder: StreamFn<Terminal, Message>,
}

impl<Terminal, Message: 'static> Subscription<Terminal, Message>
where
    Terminal: terminal::Terminal,
{
    pub fn new<S>(f: fn() -> S) -> Self
    where
        S: futures::Stream<Item = Message> + 'static,
    {
        let id: u64 = f as usize as u64;
        let builder = Box::new(move |_: &mut Terminal| f().boxed_local());
        Self { id, builder }
    }

    pub fn with<I, S>(data: I, f: fn(&I) -> S) -> Self
    where
        I: Hash + 'static,
        S: futures::Stream<Item = Message> + 'static,
    {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);

        let id: u64 = f as usize as u64 ^ hasher.finish();
        let builder = Box::new(move |_: &mut Terminal| f(&data).boxed_local());
        Self { id, builder }
    }

    pub fn key<F>(f: F) -> Self
    where
        F: Fn(KeyEvent) -> Option<Message> + 'static,
    {
        struct KeyListener;
        let mut hasher = DefaultHasher::new();
        TypeId::of::<KeyListener>().hash(&mut hasher);
        let id = hasher.finish();

        let builder = Box::new(move |terminal: &mut Terminal| {
            terminal
                .subscribe()
                .filter_map(move |res| {
                    future::ready(match res {
                        Ok(val) => match val {
                            event::Event::Key(key_event) => f(key_event),
                            _ => None,
                        },
                        Err(_) => None,
                    })
                })
                .boxed_local()
        });

        Self { id, builder }
    }

    pub fn build(self, terminal: &mut Terminal) -> Source<Message> {
        let stream = (self.builder)(terminal);
        Source::new(self.id, stream)
    }
}

pub struct Source<Message> {
    id: u64,
    stream: LocalBoxStream<'static, Message>,
}

impl<Message> Source<Message> {
    pub fn new(id: u64, stream: LocalBoxStream<'static, Message>) -> Self {
        Self { id, stream }
    }
}

impl<Message> Stream for Source<Message> {
    type Item = Message;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.stream.as_mut().poll_next(cx)
    }
}
