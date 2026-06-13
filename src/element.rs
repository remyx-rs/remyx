use crate::runner::Shell;
use crossterm::event::Event;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::text::{Span, Text};
use ratatui_core::widgets::Widget;
use ratatui_widgets::barchart::BarChart;
use ratatui_widgets::block::Block;
use ratatui_widgets::canvas::{Canvas, Context};
use ratatui_widgets::chart::Chart;
use ratatui_widgets::clear::Clear;
use ratatui_widgets::gauge::{Gauge, LineGauge};
use ratatui_widgets::paragraph::Paragraph;
use ratatui_widgets::sparkline::Sparkline;
use ratatui_widgets::tabs::Tabs;
use std::any::{Any, TypeId};
use std::cell::RefCell;

pub mod container;
pub mod list;

pub type GenericState = RefCell<Box<dyn Any>>;

#[derive(Debug)]
pub struct Tree {
    id: TypeId,
    state: Option<GenericState>,
    children: Vec<Tree>,
}

impl Tree {
    pub fn init<Message>(element: &dyn Element<Message>) -> Self {
        Self {
            id: element.id(),
            state: element.state(),
            children: element
                .children()
                .iter()
                .map(|child| Tree::init(&**child))
                .collect(),
        }
    }
    pub fn with_state<S, F, O>(&self, f: F) -> O
    where
        S: 'static,
        F: FnOnce(&S) -> O,
    {
        let state = self.state.as_ref().expect("tree has no state").borrow();

        let state = state
            .downcast_ref::<S>()
            .expect("tree state has wrong type");

        f(state)
    }

    pub fn with_state_mut<S, F, O>(&self, f: F) -> O
    where
        S: 'static,
        F: FnOnce(&mut S) -> O,
    {
        let mut state = self.state.as_ref().expect("tree has no state").borrow_mut();

        let state = state
            .downcast_mut::<S>()
            .expect("tree state has wrong type");

        f(state)
    }

    pub fn diff<Message>(&mut self, element: &dyn Element<Message>) {
        if !self.id.eq(&element.id()) {
            *self = Tree::init(element);
        } else {
            // Remove extra old children
            self.children.truncate(element.children().len());

            // Diff existing children
            element
                .children()
                .iter()
                .zip(self.children.iter_mut())
                .for_each(|(child, tree)| {
                    tree.diff(&**child);
                });

            // Add missing new children
            self.children.extend(
                element
                    .children()
                    .iter()
                    .skip(self.children.len())
                    .map(|child| Tree::init(&**child)),
            );
        }
    }
}

pub trait Element<Message> {
    fn draw(&self, tree: &Tree, area: Rect, buffer: &mut Buffer);

    fn update(&self, _tree: &Tree, _area: Rect, _event: Event, _shell: &mut Shell<Message>) {}

    fn id(&self) -> TypeId;

    fn state(&self) -> Option<GenericState> {
        None
    }

    fn children(&self) -> &[Box<dyn Element<Message>>] {
        &[]
    }
}

macro_rules! impl_stateless_element {
    ($ty:ty) => {
        impl<Message: 'static> Element<Message> for $ty {
            fn draw(&self, _tree: &Tree, area: Rect, buffer: &mut Buffer) {
                self.render(area, buffer);
            }

            fn id(&self) -> TypeId {
                TypeId::of::<$ty>()
            }
        }
    };
}

// base widgets
impl_stateless_element!(Block<'_>);
impl_stateless_element!(BarChart<'_>);
impl_stateless_element!(Chart<'_>);
impl_stateless_element!(Clear);
impl_stateless_element!(Gauge<'_>);
impl_stateless_element!(LineGauge<'_>);
impl_stateless_element!(Paragraph<'_>);
impl_stateless_element!(Sparkline<'_>);
impl_stateless_element!(Tabs<'_>);

impl<'a, Message: 'static, F> Element<Message> for Canvas<'a, F>
where
    F: Fn(&mut Context<'_>),
{
    fn draw(&self, _tree: &Tree, area: Rect, buffer: &mut Buffer) {
        self.render(area, buffer);
    }

    fn id(&self) -> TypeId {
        TypeId::of::<Canvas<'static, fn(&mut Context<'_>)>>()
    }
}

// text primitives
impl_stateless_element!(Span<'_>);
impl_stateless_element!(Text<'_>);

// string widgets
impl_stateless_element!(&str);
