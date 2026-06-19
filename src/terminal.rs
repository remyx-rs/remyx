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
pub struct Size {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    position: Position,
    terminal_size: Size,
}

impl Cursor {
    pub(crate) fn new(position: Position, terminal_size: Size) -> Self {
        Self {
            position,
            terminal_size,
        }
    }

    pub fn is_hovering(&self, area: Rect) -> bool {
        let at_terminal_edge = self.position.x == 0
            || self.position.y == 0
            || self.position.x >= self.terminal_size.width.saturating_sub(1)
            || self.position.y >= self.terminal_size.height.saturating_sub(1);

        if at_terminal_edge {
            return false;
        }

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
