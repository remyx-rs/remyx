use std::any::TypeId;

use crossterm::event::Event;
use ratatui_core::{
    buffer::Buffer,
    layout::{Position, Rect},
    widgets::StatefulWidget,
};
use remyx_widgets::paragraph::{Axe, Paragraph, ParagraphState};

use crate::{
    element::{Element, State, Tree},
    runner::Context,
};

impl<Message> Element<Message> for Paragraph<'_> {
    fn draw(&self, tree: &Tree, area: Rect, buffer: &mut Buffer) {
        tree.state_mut::<ParagraphState, _, _>(|s| {
            self.render(area, buffer, s);
        });
    }

    fn update(&self, tree: &Tree, area: Rect, event: Event, ctx: &mut Context<Message>) {
        if !ctx.cursor().is_hovering(area) {
            return;
        }

        if !tree.state::<ParagraphState, _, _>(|s| s.limits_set()) {
            tree.state_mut::<ParagraphState, _, _>(|s| {
                let limits = get_limits(self, area);
                s.limits(limits);
            });
            ctx.redraw();
        }

        enum Scroll {
            Up,
            Down,
            Left,
            Right,
        }

        let scroll = match event {
            Event::Key(key_event) => match key_event.code {
                crossterm::event::KeyCode::Up => Some(Scroll::Up),
                crossterm::event::KeyCode::Down => Some(Scroll::Down),
                crossterm::event::KeyCode::Left => Some(Scroll::Left),
                crossterm::event::KeyCode::Right => Some(Scroll::Right),
                _ => None,
            },
            Event::Mouse(mouse_event) => match mouse_event.kind {
                crossterm::event::MouseEventKind::ScrollUp => Some(Scroll::Up),
                crossterm::event::MouseEventKind::ScrollDown => Some(Scroll::Down),
                _ => None,
            },
            Event::Resize(..) => {
                tree.state_mut::<ParagraphState, _, _>(|s| {
                    let limits = get_limits(self, area);
                    s.limits(limits);
                });
                ctx.redraw();
                None
            }
            _ => None,
        };

        if let Some(scroll) = scroll {
            tree.state_mut::<ParagraphState, _, _>(|s| match scroll {
                Scroll::Up => s.offset_add(Axe::Y, -1),
                Scroll::Down => s.offset_add(Axe::Y, 1),
                Scroll::Left => s.offset_add(Axe::X, -1),
                Scroll::Right => s.offset_add(Axe::X, 1),
            });

            ctx.redraw();
        }
    }

    fn diff(&self, tree: &mut Tree) {
        let length = tree.state::<ParagraphState, _, _>(|s| s.len());
        if self.len() != length {
            tree.state = Element::<Message>::state(self);
        }
    }

    fn state(&self) -> Option<State> {
        let length = self.len();
        Some(State::new(ParagraphState::new(length)))
    }
    fn id(&self) -> std::any::TypeId {
        TypeId::of::<Paragraph<'static>>()
    }
}

fn get_limits(paragraph: &Paragraph<'_>, area: Rect) -> Position {
    Position {
        x: paragraph.line_width().saturating_sub(area.width as usize) as u16,
        y: paragraph
            .line_count(area.width)
            .saturating_sub(area.height as usize) as u16,
    }
}
