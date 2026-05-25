use std::any::TypeId;

use crossterm::event::Event;
use ratatui::{
    buffer::Buffer,
    layout::{Layout, Rect},
};

use crate::{
    Shell,
    element::{Element, GenericState, Tree},
};

pub struct Container<Message> {
    layout: Layout,
    children: Vec<Box<dyn Element<Message>>>,
}

impl<M> Container<M> {
    pub fn with_layout(layout: Layout) -> Self {
        Self {
            layout,
            children: vec![],
        }
    }
    pub fn with_child(mut self, child: impl Element<M> + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }
}

impl<Message: 'static> Element<Message> for Container<Message> {
    fn draw(&self, tree: &Tree, area: Rect, buffer: &mut Buffer) {
        self.layout
            .split(area)
            .iter()
            .zip(self.children.iter())
            .for_each(|(area, child)| {
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

    fn update(&self, tree: &Tree, event: Event, shell: &mut Shell<'_, Message>) {
        self.children
            .iter()
            .zip(tree.children.iter())
            .for_each(|(child, tree)| child.update(tree, event.clone(), shell));
    }
}
