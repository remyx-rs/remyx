use std::{
    io::{self, Stdout},
    pin::Pin,
    task::Poll,
};

use crate::{
    runtime::{self},
    stream,
    terminal::{Cursor, EventResult, Terminal},
};
use crossterm::event::{Event, EventStream};
use futures::{Stream, StreamExt, TryStreamExt, stream::FusedStream};
use ratatui_core::{
    layout::Position,
    terminal::{self, CompletedFrame, Frame},
};
use ratatui_crossterm::CrosstermBackend;

pub struct Crossterm<Runtime>
where
    Runtime: runtime::Runtime,
{
    inner: terminal::Terminal<CrosstermBackend<Stdout>>,
    event_stream: stream::Tee<Runtime, Result<Event, io::ErrorKind>>,
    mouse_pos: Position,
}

impl<Runtime> Default for Crossterm<Runtime>
where
    Runtime: runtime::Runtime,
 {
    fn default() -> Self {
        Self::new()
    }
}

impl<Runtime> Crossterm<Runtime>
where
    Runtime: runtime::Runtime,
{
    pub fn new() -> Self {
        let backend = CrosstermBackend::new(io::stdout());
        let inner = terminal::Terminal::new(backend)
            .expect("failed to create terminal using Crossterm backend");
        let stream = EventStream::new().map_err(|err| err.kind()).boxed();
        Self {
            inner,
            event_stream: stream::Tee::new(stream),
            mouse_pos: Position { x: 0, y: 0 },
        }
    }
}

impl<Runtime> Terminal for Crossterm<Runtime>
where
    Runtime: runtime::Runtime,
{
    fn mouse(&self) -> Cursor {
        Cursor::new(self.mouse_pos)
    }

    fn subscribe(&mut self) -> impl Stream<Item = EventResult> + 'static {
        self.event_stream.fork()
    }

    fn get_frame(&mut self) -> Frame<'_> {
        self.inner.get_frame()
    }

    fn draw<F>(&mut self, render_callback: F) -> io::Result<CompletedFrame<'_>>
    where
        F: FnOnce(&mut Frame),
    {
        self.inner.draw(render_callback)
    }
}

impl<Runtime> Stream for Crossterm<Runtime>
where
    Runtime: runtime::Runtime,
{
    type Item = EventResult;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = &mut self.event_stream;
        let result = Pin::new(this).poll_next(cx);

        if let Poll::Ready(Some(Ok(Event::Mouse(mouse)))) = &result {
            self.mouse_pos = Position {
                x: mouse.column,
                y: mouse.row,
            };
        }

        result
    }
}

impl<Runtime> FusedStream for Crossterm<Runtime>
where
    Runtime: runtime::Runtime,
{
    fn is_terminated(&self) -> bool {
        self.event_stream.is_terminated()
    }
}
