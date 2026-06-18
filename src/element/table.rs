use std::any::TypeId;

use crate::{
    element::{Element, State, Tree},
    runner::Context,
};
use ratatui_core::widgets::StatefulWidget;
use ratatui_core::{buffer::Buffer, layout::Rect};
use remyx_widgets::table::{Row, Table, TableState};

impl<Item, Message> Element<Message> for Table<'static, Item, Message>
where
    Message: 'static,
    Item: Clone + Into<Row<'static>> + 'static,
{
    fn draw(&self, tree: &Tree, area: Rect, buffer: &mut Buffer) {
        tree.state_mut::<TableState, _, _>(|s| {
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
        // Rows are navigated vertically and columns horizontally. Only a row change selects a
        // different `Item` reported through `on_select`.
        enum Movement {
            RowPrevious,
            RowNext,
            RowAt(usize),
            ColumnPrevious,
            ColumnNext,
        }

        if !ctx.cursor().is_hovering(area) {
            return;
        }

        let items_area = self.items_layout(area);
        let movement = match event {
            crossterm::event::Event::Key(key_event) => match key_event.code {
                crossterm::event::KeyCode::Up => Some(Movement::RowPrevious),
                crossterm::event::KeyCode::Down => Some(Movement::RowNext),
                crossterm::event::KeyCode::Left => Some(Movement::ColumnPrevious),
                crossterm::event::KeyCode::Right => Some(Movement::ColumnNext),
                _ => None,
            },
            crossterm::event::Event::Mouse(mouse_event) => match mouse_event.kind {
                crossterm::event::MouseEventKind::Up(mouse_button)
                    if mouse_button.eq(&crossterm::event::MouseButton::Left) =>
                {
                    ctx.cursor()
                        .is_hovering(items_area)
                        .then(|| {
                            // Rows can span several lines (height + margins), so walk them from the
                            // offset accumulating spans until the total passes the clicked line.
                            let offset = tree.state::<TableState, _, _>(|s| s.offset());
                            let click_position = (mouse_event.row - items_area.y) as usize;

                            let mut item_height = 0usize;
                            self.row_heights().enumerate().skip(offset).find_map(
                                |(index, height)| {
                                    item_height += height as usize;
                                    (click_position < item_height).then_some(Movement::RowAt(index))
                                },
                            )
                        })
                        .flatten()
                }
                crossterm::event::MouseEventKind::ScrollUp => Some(Movement::RowPrevious),
                crossterm::event::MouseEventKind::ScrollDown => Some(Movement::RowNext),
                _ => None,
            },
            _ => None,
        };

        if let Some(movement) = movement {
            tree.state_mut::<TableState, _, _>(|s| {
                let row_changed = match movement {
                    Movement::RowPrevious => {
                        s.select_previous();
                        true
                    }
                    Movement::RowNext => {
                        s.select_next();
                        true
                    }
                    Movement::RowAt(index) => {
                        s.select(Some(index));
                        true
                    }
                    Movement::ColumnPrevious => {
                        s.select_previous_column();
                        false
                    }
                    Movement::ColumnNext => {
                        s.select_next_column();
                        false
                    }
                };

                ctx.redraw();

                // Publish the callback only when the selected row changed.
                if row_changed
                    && let Some(f) = self.on_select_ref()
                    && let Some(item_index) = s.selected()
                    && let Some(item) = self.items_as_slice().get(item_index)
                {
                    ctx.publish(f(item));
                }
            });
        }
    }

    fn id(&self) -> TypeId {
        TypeId::of::<Table<'static, Item, Message>>()
    }

    fn state(&self) -> Option<State> {
        Some(State::new(TableState::default()))
    }
}
