use std::{any::TypeId, cell::RefCell};

use crossterm::event::Event;
use ratatui_core::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};
use remyx_widgets::paragraph::{Paragraph, ParagraphState};

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

        enum Scroll {
            Up,
            Down,
        }

        let scroll = match event {
            Event::Key(key_event) => match key_event.code {
                crossterm::event::KeyCode::Up => Some(Scroll::Up),
                crossterm::event::KeyCode::Down => Some(Scroll::Down),
                _ => None,
            },
            Event::Mouse(mouse_event) => match mouse_event.kind {
                crossterm::event::MouseEventKind::ScrollUp => Some(Scroll::Up),
                crossterm::event::MouseEventKind::ScrollDown => Some(Scroll::Down),
                _ => None,
            },
            _ => None,
        };

        if let Some(scroll) = scroll {
            tree.with_state_mut(|state: &mut ParagraphState| match scroll {
                Scroll::Up => state.decr_offset_y(),
                Scroll::Down => state.incr_offset_y(),
            });
            ctx.redraw();
        }
    }

    fn state(&self) -> Option<GenericState> {
        Some(RefCell::new(Box::new(ParagraphState::new())))
    }

    fn id(&self) -> std::any::TypeId {
        TypeId::of::<Paragraph<'static>>()
    }
}
