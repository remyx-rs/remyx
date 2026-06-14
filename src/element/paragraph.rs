use std::{any::TypeId, cell::RefCell};

use crossterm::event::Event;
use ratatui_core::{
    buffer::Buffer,
    layout::{Position, Rect},
    widgets::StatefulWidget,
};
use remyx_widgets::paragraph::{Axe, Paragraph, ParagraphState};

use crate::{
    element::{Element, GenericState, Tree},
    runner::Context,
};

impl<Message> Element<Message> for Paragraph<'_> {
    fn draw(&self, tree: &Tree, area: Rect, buffer: &mut Buffer) {
        tree.with_state_mut(|state: &mut ParagraphState| {
            self.render(area, buffer, state);
        });
    }

    fn update(&self, tree: &Tree, area: Rect, event: Event, ctx: &mut Context<Message>) {
        if !ctx.cursor().is_hovering(area) {
            return;
        }

        if !tree.with_state(|s: &ParagraphState| s.limits_set()) {
            tree.with_state_mut(|state: &mut ParagraphState| {
                let limits = get_limits(self, area);
                state.limits(limits);
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
                tree.with_state_mut(|state: &mut ParagraphState| {
                    let limits = get_limits(self, area);
                    state.limits(limits);
                });
                ctx.redraw();
                None
            }
            _ => None,
        };

        if let Some(scroll) = scroll {
            tree.with_state_mut(|state: &mut ParagraphState| match scroll {
                Scroll::Up => state.offset_add(Axe::Y, -1),
                Scroll::Down => state.offset_add(Axe::Y, 1),
                Scroll::Left => state.offset_add(Axe::X, -1),
                Scroll::Right => state.offset_add(Axe::X, 1),
            });

            ctx.redraw();
        }
    }

    fn diff(&self, tree: &mut Tree) {
        let length = tree.with_state(|s: &ParagraphState| s.len());
        if self.len() != length {
            tree.state = Element::<Message>::state(self);
        }
    }

    fn state(&self) -> Option<GenericState> {
        let length = self.len();
        Some(RefCell::new(Box::new(ParagraphState::new(length))))
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
