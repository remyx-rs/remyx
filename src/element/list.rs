use std::any::TypeId;

use crate::{
    element::{Element, State, Tree},
    runner::Context,
};
use crossterm::event::{Event, MouseButton};
use ratatui_core::widgets::StatefulWidget;
use ratatui_core::{buffer::Buffer, layout::Rect};
use remyx_widgets::{
    focus::Focusable,
    list::{List, ListDirection, ListItem, ListState},
};

impl<Item, Message> Element<Message> for List<'static, Item, Message>
where
    Message: 'static,
    Item: Clone + Into<ListItem<'static>> + 'static,
{
    fn draw(&self, tree: &Tree, area: Rect, buffer: &mut Buffer) {
        tree.state_mut::<ListState, _, _>(|s| {
            self.render(area, buffer, s);
        });
    }

    fn update(
        &self,
        tree: &Tree,
        area: Rect,
        event: crossterm::event::Event,
        ctx: &mut Context<Message>,
    ) {
        enum Selection {
            Previous,
            Next,
            Index(usize),
        }

        if !ctx.cursor().is_hovering(area) && !self.is_focused() {
            return;
        }

        let items_area = self.items_layout(area);
        let offset = tree.state::<ListState, _, _>(|s| s.offset());
        let selection = match self.direction_ref() {
            ListDirection::TopToBottom => match event {
                crossterm::event::Event::Key(key_event) => match key_event.code {
                    crossterm::event::KeyCode::Up => Some(Selection::Previous),
                    crossterm::event::KeyCode::Down => Some(Selection::Next),
                    _ => None,
                },
                Event::Mouse(mouse_event) => match mouse_event.kind {
                    crossterm::event::MouseEventKind::Up(mouse_button)
                        if mouse_button.eq(&MouseButton::Left) =>
                    {
                        ctx.cursor()
                            .is_hovering(items_area)
                            .then(|| {
                                let item_index = (mouse_event.row - items_area.y) as usize + offset;
                                (item_index < self.len()).then_some(Selection::Index(item_index))
                            })
                            .flatten()
                    }
                    crossterm::event::MouseEventKind::ScrollUp => Some(Selection::Previous),
                    crossterm::event::MouseEventKind::ScrollDown => Some(Selection::Next),
                    _ => None,
                },
                _ => None,
            },
            ListDirection::BottomToTop => match event {
                Event::Key(key_event) => match key_event.code {
                    crossterm::event::KeyCode::Up => Some(Selection::Next),
                    crossterm::event::KeyCode::Down => Some(Selection::Previous),
                    _ => None,
                },
                Event::Mouse(mouse_event) => match mouse_event.kind {
                    crossterm::event::MouseEventKind::Up(mouse_button)
                        if mouse_button.eq(&MouseButton::Left) =>
                    {
                        ctx.cursor()
                            .is_hovering(items_area)
                            .then(|| {
                                let item_index =
                                    (items_area.y + items_area.height - 1 - mouse_event.row)
                                        as usize
                                        + offset;
                                (item_index < self.len()).then_some(Selection::Index(item_index))
                            })
                            .flatten()
                    }
                    crossterm::event::MouseEventKind::ScrollUp => Some(Selection::Next),
                    crossterm::event::MouseEventKind::ScrollDown => Some(Selection::Previous),
                    _ => None,
                },
                _ => None,
            },
        };

        if let Some(selection) = selection {
            tree.state_mut::<ListState, _, _>(|s| {
                match selection {
                    Selection::Previous => s.select_previous(),
                    Selection::Next => s.select_next(),
                    Selection::Index(index) => *s = s.with_selected(Some(index)),
                }

                ctx.redraw();
                if let Some(f) = self.on_select_ref()
                    && let Some(item_index) = s.selected()
                    && let Some(item) = self.items_as_slice().get(item_index)
                {
                    ctx.publish(f(item));
                }
            });
        }
    }

    fn diff(&self, tree: &mut Tree) {
        let length = tree.state::<ListState, _, _>(|s| s.len());
        if self.len() != length {
            tree.state = Element::<Message>::state(self);
        }
    }

    fn id(&self) -> TypeId {
        TypeId::of::<List<'static, Item, Message>>()
    }

    fn state(&self) -> Option<State> {
        let length = self.len();
        Some(State::new(ListState::new(length)))
    }
}
