use crate::runner::Context;
use crossterm::event::Event;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::text::{Span, Text};
use ratatui_core::widgets::Widget;
use remyx_widgets::barchart::BarChart;
use remyx_widgets::block::Block;
use remyx_widgets::canvas::{Canvas, Context as CanvasContext};
use remyx_widgets::chart::Chart;
use remyx_widgets::clear::Clear;
use remyx_widgets::fill::Fill;
use remyx_widgets::gauge::{Gauge, LineGauge};
use remyx_widgets::logo::RatatuiLogo;
use remyx_widgets::mascot::RatatuiMascot;
use remyx_widgets::sparkline::Sparkline;
use remyx_widgets::tabs::Tabs;
use std::any::{Any, TypeId};
use std::cell::RefCell;

pub mod container;
pub mod list;
pub mod paragraph;
pub mod table;

#[derive(Debug)]
pub struct State(RefCell<Box<dyn Any>>);

impl State {
    pub fn new<State: Any>(state: State) -> Self {
        Self(RefCell::new(Box::new(state)))
    }
}

#[derive(Debug)]
pub struct Tree {
    id: TypeId,
    state: Option<State>,
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
    pub fn state<S, F, O>(&self, f: F) -> O
    where
        S: 'static,
        F: FnOnce(&S) -> O,
    {
        let generic = self.state.as_ref().expect("tree has no state").0.borrow();
        let state = generic
            .downcast_ref::<S>()
            .expect("tree state has wrong type");

        f(state)
    }

    pub fn state_mut<S, F, O>(&self, f: F) -> O
    where
        S: 'static,
        F: FnOnce(&mut S) -> O,
    {
        let mut generic = self
            .state
            .as_ref()
            .expect("tree has no state")
            .0
            .borrow_mut();

        let state = generic
            .downcast_mut::<S>()
            .expect("tree state has wrong type");

        f(state)
    }

    pub fn diff<Message>(&mut self, element: &dyn Element<Message>) {
        if !self.id.eq(&element.id()) {
            *self = Tree::init(element);
        } else {
            // Check State compatibility
            element.diff(self);

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

    fn update(&self, _tree: &Tree, _area: Rect, _event: Event, _ctx: &mut Context<Message>) {}

    fn id(&self) -> TypeId;

    fn state(&self) -> Option<State> {
        None
    }

    fn diff(&self, _tree: &mut Tree) {}

    fn children(&self) -> &[Box<dyn Element<Message>>] {
        &[]
    }
}

macro_rules! impl_stateless_element {
    ($ty:ty) => {
        impl<Message> Element<Message> for $ty {
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
impl_stateless_element!(Fill<'_>);
impl_stateless_element!(Gauge<'_>);
impl_stateless_element!(LineGauge<'_>);
impl_stateless_element!(Sparkline<'_>);
impl_stateless_element!(Tabs<'_>);
impl_stateless_element!(RatatuiLogo);
impl_stateless_element!(RatatuiMascot);

impl<'a, Message, F> Element<Message> for Canvas<'a, F>
where
    F: Fn(&mut CanvasContext<'_>),
{
    fn draw(&self, _tree: &Tree, area: Rect, buffer: &mut Buffer) {
        self.render(area, buffer);
    }

    fn id(&self) -> TypeId {
        TypeId::of::<Canvas<'static, fn(&mut CanvasContext<'_>)>>()
    }
}

// text primitives
impl_stateless_element!(Span<'_>);
impl_stateless_element!(Text<'_>);

// string widgets
impl_stateless_element!(&str);
