use ::crossterm::event::Event;
use futures::{Stream, stream::FusedStream};
use ratatui_core::{
    layout::{Position, Rect},
    terminal::{CompletedFrame, Frame},
};
use std::io::{self};

pub mod crossterm;

pub type EventResult = Result<Event, io::ErrorKind>;

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    position: Position,
}

impl Cursor {
    pub(crate) fn new(position: Position) -> Self {
        Self { position }
    }

    pub fn is_hovering(&self, area: Rect) -> bool {
        self.position.x >= area.x
            && self.position.x < area.x + area.width
            && self.position.y >= area.y
            && self.position.y < area.y + area.height
    }
}

pub trait Terminal: FusedStream<Item = EventResult> + Unpin {
    fn mouse(&self) -> Cursor;
    fn subscribe(&mut self) -> impl Stream<Item = EventResult> + 'static;
    fn get_frame(&mut self) -> Frame<'_>;
    fn draw<F>(&mut self, render_callback: F) -> io::Result<CompletedFrame<'_>>
    where
        F: FnOnce(&mut Frame);
}
