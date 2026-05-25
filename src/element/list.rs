use std::{any::TypeId, cell::RefCell};

use crossterm::event::MouseButton;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{List, ListItem, ListState, StatefulWidget},
};

use crate::{
    Shell,
    element::{Element, GenericState, Tree},
};

pub struct PickList<Message, Item> {
    list: Box<dyn Element<Message>>,
    items: Vec<Item>,
    on_select: Option<fn(&Item) -> Option<Message>>,
}

impl<Message: 'static, Item: 'static> Element<Message> for PickList<Message, Item> {
    fn draw(&self, tree: &Tree, area: Rect, buffer: &mut Buffer) {
        self.list.draw(tree, area, buffer);
    }

    fn update(
        &self,
        tree: &Tree,
        area: Rect,
        event: crossterm::event::Event,
        shell: &mut Shell<'_, Message>,
    ) {
        let previous = tree.with_state(|state: &State| state.list.selected());
        self.list.update(tree, area, event, shell);
        let next = tree.with_state(|state: &State| state.list.selected());

        if let Some(f) = self.on_select
            && let Some(msg) = match (previous, next) {
                (None, Some(next)) => f(self.items.get(next).unwrap()),
                (Some(previous), Some(next)) if previous != next => {
                    f(self.items.get(next).unwrap())
                }
                _ => None,
            }
        {
            shell.publish(msg);
        }
    }

    fn id(&self) -> TypeId {
        TypeId::of::<PickList<Message, Item>>()
    }

    fn state(&self) -> Option<GenericState> {
        self.list.state()
    }

    fn children(&self) -> &[Box<dyn Element<Message>>] {
        &[]
    }
}

impl<Message, Item> PickList<Message, Item> {
    pub fn list<F>(items: Vec<Item>, f: F) -> Self
    where
        F: FnOnce(List) -> List,
        Item: Into<ListItem<'static>> + Clone,
    {
        // TODO: Check
        let list: Box<dyn Element<Message>> = Box::new(f(List::new(items.clone())));
        Self {
            list,
            items,
            on_select: None,
        }
    }

    pub fn on_select(mut self, f: fn(&Item) -> Option<Message>) -> Self {
        self.on_select = Some(f);
        self
    }
}

pub struct State {
    list: ListState,
    hovered: bool,
}

impl<Message> Element<Message> for List<'_> {
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
        shell: &mut Shell<'_, Message>,
    ) {
        match event {
            crossterm::event::Event::Key(key_event)
                if tree.with_state(|state: &State| state.hovered) =>
            {
                match key_event.code {
                    crossterm::event::KeyCode::Up => {
                        tree.with_state_mut(|state: &mut State| {
                            state.list.select_previous();
                        });
                        shell.request_redraw();
                    }
                    crossterm::event::KeyCode::Down => {
                        tree.with_state_mut(|state: &mut State| {
                            state.list.select_next();
                        });
                        shell.request_redraw();
                    }

                    _ => {}
                }
            }

            crossterm::event::Event::Mouse(mouse_event) => {
                tree.with_state_mut(|state: &mut State| {
                    state.hovered = mouse_event.column >= area.x
                        && mouse_event.column < (area.x + area.width)
                        && mouse_event.row >= area.y
                        && mouse_event.row < (area.y + area.height);
                });

                if !tree.with_state(|state: &State| state.hovered) {
                    return;
                }

                match mouse_event.kind {
                    crossterm::event::MouseEventKind::Up(mouse_button)
                        if mouse_button.eq(&MouseButton::Left) =>
                    {
                        let selected = (mouse_event.row - area.y) as usize;
                        if selected > self.len() - 1 {
                            return;
                        }

                        tree.with_state_mut(|state: &mut State| {
                            state.list = state.list.with_selected(Some(selected))
                        });
                        shell.request_redraw();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn id(&self) -> std::any::TypeId {
        TypeId::of::<List>()
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
