use std::{any::TypeId, cell::RefCell};

use crate::{
    element::{Element, GenericState, Tree},
    runner::Shell,
};
use crossterm::event::MouseButton;
use ratatui_core::widgets::StatefulWidget;
use ratatui_core::{buffer::Buffer, layout::Rect};
use ratatui_widgets::list::{List, ListItem, ListState};

impl<Item, Message> Element<Message> for List<'static, Item, Message>
where
    Message: 'static,
    Item: Clone + Into<ListItem<'static>> + 'static,
{
    fn draw(&self, tree: &Tree, area: Rect, buffer: &mut Buffer) {
        tree.with_state_mut(|state: &mut State| {
            self.render(area, buffer, &mut state.list);
        });
    }

    fn update(
        &self,
        tree: &Tree,
        area: Rect,
        event: crossterm::event::Event,
        shell: &mut Shell<Message>,
    ) {
        let items_area = if let Some(block) = self.block_as_ref() {
            block.inner(area)
        } else {
            area
        };

        match event {
            crossterm::event::Event::Key(key_event)
                if tree.with_state(|state: &State| state.hovered) =>
            {
                match key_event.code {
                    crossterm::event::KeyCode::Up => {
                        tree.with_state_mut(|state: &mut State| {
                            state.list.select_previous();

                            if let Some(f) = self.on_select_ref()
                                && let Some(item_index) = state.list.selected()
                                && let Some(item) = self.items_as_slice().get(item_index)
                            {
                                shell.publish(f(item));
                            }
                        });

                        shell.request_redraw();
                    }
                    crossterm::event::KeyCode::Down => {
                        tree.with_state_mut(|state: &mut State| {
                            state.list.select_next();

                            if let Some(f) = self.on_select_ref()
                                && let Some(item_index) = state.list.selected()
                                && let Some(item) = self.items_as_slice().get(item_index)
                            {
                                shell.publish(f(item));
                            }
                        });
                        shell.request_redraw();
                    }

                    _ => {}
                }
            }
            crossterm::event::Event::Mouse(mouse_event) => {
                tree.with_state_mut(|state: &mut State| {
                    state.hovered = mouse_event.column >= items_area.x
                        && mouse_event.column < (items_area.x + items_area.width)
                        && mouse_event.row >= items_area.y
                        && mouse_event.row < (items_area.y + items_area.height);
                });

                if !tree.with_state(|state: &State| state.hovered) {
                    return;
                }

                match mouse_event.kind {
                    crossterm::event::MouseEventKind::Up(mouse_button)
                        if mouse_button.eq(&MouseButton::Left) =>
                    {
                        let item_index = (mouse_event.row - items_area.y) as usize;
                        if item_index >= self.len() {
                            return;
                        }

                        tree.with_state_mut(|state: &mut State| {
                            state.list = state.list.with_selected(Some(item_index));

                            if let Some(f) = self.on_select_ref()
                                && let Some(item_index) = state.list.selected()
                                && let Some(item) = self.items_as_slice().get(item_index)
                            {
                                shell.publish(f(item));
                            }
                        });

                        shell.request_redraw();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn id(&self) -> TypeId {
        TypeId::of::<List<'static, Item, Message>>()
    }

    fn state(&self) -> Option<GenericState> {
        Some(RefCell::new(Box::new(State {
            list: ListState::default(),
            hovered: false,
        })))
    }

    fn children(&self) -> &[Box<dyn Element<Message>>] {
        &[]
    }
}

pub struct State {
    list: ListState,
    hovered: bool,
}
