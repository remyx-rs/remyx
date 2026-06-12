use std::any::TypeId;

use crossterm::event::Event;
use ratatui_core::{
    buffer::Buffer,
    layout::{Layout, Rect},
};

use crate::{
    element::{Element, GenericState, Tree},
    runner::Shell,
};

pub struct Container<Message> {
    layout: Layout,
    children: Vec<Box<dyn Element<Message>>>,
}

impl<Message> Container<Message> {
    pub fn layout(layout: Layout) -> Self {
        Self {
            layout,
            children: vec![],
        }
    }

    pub fn with(mut self, child: impl Element<Message> + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }
}

impl<Message: 'static> Element<Message> for Container<Message> {
    fn draw(&self, tree: &Tree, area: Rect, buffer: &mut Buffer) {
        self.layout
            .split(area)
            .iter()
            .zip(tree.children.iter())
            .zip(self.children.iter())
            .for_each(|((area, tree), child)| {
                child.draw(tree, *area, buffer);
            });
    }

    fn id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn state(&self) -> Option<GenericState> {
        None
    }

    fn children(&self) -> &[Box<dyn Element<Message>>] {
        self.children.as_slice()
    }

    fn update(&self, tree: &Tree, area: Rect, event: Event, shell: &mut Shell<Message>) {
        self.layout
            .split(area)
            .iter()
            .zip(tree.children.iter())
            .zip(self.children.iter())
            .for_each(|((area, tree), child)| {
                child.update(tree, *area, event.clone(), shell);
            });
    }
}
