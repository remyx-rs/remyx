use std::{any::TypeId, cell::RefCell};

use crate::{
    element::{Element, GenericState, Tree},
    runner::Shell,
};
use crossterm::event::MouseButton;
use ratatui_core::widgets::StatefulWidget;
use ratatui_core::{buffer::Buffer, layout::Rect, style::Style, text::Line};
use ratatui_widgets::{
    block::Block,
    list::{List, ListDirection, ListItem, ListState},
    table::HighlightSpacing,
};

pub struct PickList<Message, Item>
where
    Item: Into<ListItem<'static>> + Clone,
{
    list: List<'static>,
    block: Option<Block<'static>>,
    direction: ListDirection,
    items: Vec<Item>,
    on_select: Option<fn(&Item) -> Message>,
}

impl<Message, Item> PickList<Message, Item>
where
    Item: Into<ListItem<'static>> + Clone,
{
    pub fn new(items: Vec<Item>) -> Self {
        Self {
            list: List::new(items.clone()),
            items,
            on_select: None,
            direction: ListDirection::default(),
            block: None,
        }
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn block(mut self, block: Block<'static>) -> Self {
        self.block = Some(block.clone());
        self.list = self.list.block(block);
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn style<S: Into<Style>>(mut self, style: S) -> Self {
        self.list = self.list.style(style.into());
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn highlight_symbol<L: Into<Line<'static>>>(mut self, highlight_symbol: L) -> Self {
        self.list = self.list.highlight_symbol(highlight_symbol.into());
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn highlight_style<S: Into<Style>>(mut self, style: S) -> Self {
        self.list = self.list.highlight_style(style.into());
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn repeat_highlight_symbol(mut self, repeat: bool) -> Self {
        self.list = self.list.repeat_highlight_symbol(repeat);
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn highlight_spacing(mut self, value: HighlightSpacing) -> Self {
        self.list = self.list.highlight_spacing(value);
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn direction(mut self, direction: ListDirection) -> Self {
        self.direction = direction;
        self.list = self.list.direction(direction);
        self
    }

    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn scroll_padding(mut self, padding: usize) -> Self {
        self.list = self.list.scroll_padding(padding);
        self
    }

    pub fn on_select(mut self, f: fn(&Item) -> Message) -> Self {
        self.on_select = Some(f);
        self
    }
}

impl<Message: 'static, Item: 'static> Element<Message> for PickList<Message, Item>
where
    Item: Into<ListItem<'static>> + Clone,
{
    fn draw(&self, tree: &Tree, area: Rect, buffer: &mut Buffer) {
        tree.with_state_mut(|state: &mut State| {
            (&self.list).render(area, buffer, &mut state.list);
        });
    }

    fn update(
        &self,
        tree: &Tree,
        area: Rect,
        event: crossterm::event::Event,
        shell: &mut Shell<Message>,
    ) {
        let items_area = if let Some(block) = self.block.as_ref() {
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

                            if let Some(on_select) = self.on_select
                                && let Some(item_index) = state.list.selected()
                            {
                                shell.publish(on_select(self.items.get(item_index).unwrap()))
                            }
                        });

                        shell.request_redraw();
                    }
                    crossterm::event::KeyCode::Down => {
                        tree.with_state_mut(|state: &mut State| {
                            state.list.select_next();

                            if let Some(on_select) = self.on_select
                                && let Some(item_index) = state.list.selected()
                            {
                                shell.publish(on_select(self.items.get(item_index).unwrap()))
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
                        if item_index >= self.list.len() {
                            return;
                        }

                        tree.with_state_mut(|state: &mut State| {
                            state.list = state.list.with_selected(Some(item_index))
                        });

                        shell.request_redraw();

                        if let Some(on_select) = self.on_select {
                            shell.publish(on_select(self.items.get(item_index).unwrap()))
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn id(&self) -> TypeId {
        TypeId::of::<PickList<Message, Item>>()
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
