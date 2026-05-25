use std::{any::TypeId, cell::RefCell};

use crossterm::event::MouseButton;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{List, ListState, StatefulWidget},
};

use crate::{
    Shell,
    element::{Element, GenericState, Tree},
};

pub struct State {
    list: ListState,
    position: Rect,
}

impl<Message> Element<Message> for List<'_> {
    fn draw(&self, tree: &Tree, area: Rect, buffer: &mut Buffer) {
        // TODO: At the moment we store the positions of each widget on the state at draw time.
        tree.with_state_mut(|state: &mut State| {
            state.position = area;
        });

        tree.with_state_mut(|state: &mut State| {
            self.render(area, buffer, &mut state.list);
        });
    }

    fn update(&self, tree: &Tree, event: crossterm::event::Event, shell: &mut Shell<'_, Message>) {
        match event {
            crossterm::event::Event::Key(key_event) => match key_event.code {
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
            },
            crossterm::event::Event::Mouse(mouse_event) => match mouse_event.kind {
                crossterm::event::MouseEventKind::Up(mouse_button)
                    if mouse_button.eq(&MouseButton::Left) =>
                {
                    // TODO:
                    let position = tree.with_state(|state: &State| state.position);
                    let (mouse_x, mouse_y) = (mouse_event.column, mouse_event.row);

                    let selected = mouse_y - position.y;

                    if selected as usize > self.len() - 1 {
                        return;
                    }

                    tree.with_state_mut(|state: &mut State| {
                        state.list = state.list.with_selected(Some(mouse_y as usize))
                    });

                    shell.request_redraw();

                    // let is_hovering = mouse_x >= position.x
                    //     && mouse_x < position.x.saturating_add(position.width)
                    //     && mouse_y >= position.y
                    //     && mouse_y < position.y.saturating_add(position.height);
                    // if is_hovering {
                    //     let item_count = self.len();
                    //     if item_count > 0 && position.height > 0 {
                    //         let local_y = mouse_y - position.y;
                    //         let visible_height = position.height as usize;

                    //         let selected = local_y as usize;
                    //         if selected < item_count.min(visible_height) {
                    //             tree.with_state_mut(|state: &mut State| {
                    //                 state.list.with_selected(Some(selected))
                    //             });

                    //             shell.request_redraw();
                    //         }
                    //     }
                    // }
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn id(&self) -> std::any::TypeId {
        TypeId::of::<List>()
    }

    fn state(&self) -> Option<GenericState> {
        Some(RefCell::new(Box::new(State {
            list: ListState::default().with_selected(Some(0)),
            position: Rect::default(),
        })))
    }

    fn children(&self) -> &[Box<dyn Element<Message>>] {
        &[]
    }
}
