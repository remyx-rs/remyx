use std::io;

use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::Backend;

use crate::{
    element::{Element, Tree},
    stream::Source,
};

pub mod element;
pub mod stream;

// TODO: Navigation by keyboard or by mouse (both?). Focus behavior
// TODO: Add execution async task and results as Stream but not execution.

enum Event<Message> {
    Terminal(crossterm::event::Event),
    Subscription(Message),
}

pub async fn run<A: Application, B: Backend>(mut terminal: Terminal<B>) -> io::Result<()> {
    let mut app = A::init();
    let mut view = app.view();
    let mut tree = Tree::init(&view);

    let _ = terminal.draw(|frame| {
        view.draw(&tree, frame.area(), frame.buffer_mut());
    });

    let mut messages = Vec::new();

    let terminal_events = stream::terminal_event().map(|res| res.map(Event::Terminal));
    let subscription_events = stream::Stream::init(app.subscription())
        .map(|res| io::Result::Ok(Event::Subscription(res)));
    let mut events = futures::stream::select(terminal_events, subscription_events);
    while let Some(Ok(item)) = events.next().await {
        match item {
            Event::Terminal(event) => match event {
                crossterm::event::Event::FocusGained
                | crossterm::event::Event::FocusLost
                | crossterm::event::Event::Key(_)
                | crossterm::event::Event::Mouse(_)
                | crossterm::event::Event::Paste(_) => {
                    let mut shell = Shell::new(&mut messages);
                    view.update(&mut tree, event, &mut shell);

                    if !shell.redraw() {
                        continue;
                    }

                    messages.drain(..).for_each(|msg| app.update(msg));

                    view = app.view();
                    tree.diff(&app.view());
                }
                crossterm::event::Event::Resize(_, _) => {
                    view = app.view();
                }
            },
            Event::Subscription(message) => {
                app.update(message);

                view = app.view();
                tree.diff(&app.view());
            }
        }

        let _ = terminal.draw(|frame| {
            view.draw(&tree, frame.area(), frame.buffer_mut());
        });
    }

    Ok(())
}

pub trait Application {
    type Message;

    fn init() -> Self;
    fn view(&self) -> impl Element<Self::Message> + use<Self>;
    fn update(&mut self, message: Self::Message);
    fn subscription(&self) -> Vec<Source<Self::Message>> {
        vec![]
    }
}

#[derive(Debug)]
pub struct Shell<'a, Message> {
    messages: &'a mut Vec<Message>,
    redraw_requested: bool,
}

impl<'a, Message> Shell<'a, Message> {
    pub fn new(messages: &'a mut Vec<Message>) -> Self {
        Self {
            messages,
            redraw_requested: false,
        }
    }

    pub fn publish(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn redraw(&self) -> bool {
        self.redraw_requested || !self.messages.is_empty()
    }

    pub fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }
}
