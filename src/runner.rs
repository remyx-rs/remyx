use futures::{FutureExt, StreamExt};
use ratatui::{Terminal, backend};
use std::{io, marker::PhantomData};

use crate::{
    element::{Element, Tree},
    runtime::{self, JoinHandle},
    stream::{self, LocalBoxFusedStream, Source},
    terminal,
};

pub struct Runner<Application, Runtime, Backend>
where
    Application: crate::Application,
    Runtime: runtime::Runtime,
    Backend: ratatui::backend::Backend,
{
    terminal: Terminal<Backend>,
    app: Application,
    tree: Tree,
    subscription_events: stream::Stream<Application::Message>,
    terminal_events: LocalBoxFusedStream<io::Result<crossterm::event::Event>>,
    messages: Vec<Application::Message>,
    _rt_marker: PhantomData<Runtime>,
}

impl<Application, Runtime, Backend> Runner<Application, Runtime, Backend>
where
    Application: crate::Application,
    Runtime: runtime::Runtime,
    Backend: backend::Backend,
{
    pub fn new(terminal: Terminal<Backend>) -> io::Result<Self> {
        let (app, task) = Application::init();
        let tree = Tree::init(&app.view());

        let mut subscriptions = app.subscription();
        if let Some(task) = task {
            let fut = Runtime::spawn(task).into_future().map(|res| match res {
                Ok(val) => val,
                Err(_) => panic!(),
            });
            subscriptions.push(Source::future(fut));
        }

        Ok(Self {
            terminal,
            app,
            tree,
            subscription_events: stream::Stream::init(subscriptions),
            terminal_events: terminal::events(),
            messages: Vec::new(),
            _rt_marker: PhantomData,
        })
    }

    pub async fn run(mut self) -> io::Result<()> {
        self.redraw();
        loop {
            futures::select_biased! {
                event = self.subscription_events.next() => match event {
                    Some(msg) => {
                        self.update(msg);
                    }
                    None => break,
                },
                event = self.terminal_events.next() => match event {
                    Some(Ok(event)) => {
                        self.handle_terminal_event(event);
                    }
                    Some(Err(e)) => return Err(e),
                    None => break,
                },
            }

            self.redraw();
        }

        Ok(())
    }

    fn update(&mut self, msg: Application::Message) {
        if let Some(task) = self.app.update(msg) {
            let fut = Runtime::spawn(task).into_future().map(|res| match res {
                Ok(val) => val,
                Err(_) => panic!(),
            });
            self.subscription_events.add(Source::future(fut));
        }
    }

    fn handle_terminal_event(&mut self, event: crossterm::event::Event) {
        let mut shell = Shell::new(&mut self.messages);
        let area = self.terminal.get_frame().area();

        self.app.view().update(&self.tree, area, event, &mut shell);

        if !shell.redraw() {
            return;
        }

        let messages = self.messages.drain(..).collect::<Vec<_>>();
        for msg in messages {
            self.update(msg);
        }
    }

    fn redraw(&mut self) {
        let view = self.app.view();
        self.tree.diff(&view);

        let _ = self.terminal.draw(|frame| {
            view.draw(&self.tree, frame.area(), frame.buffer_mut());
        });
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
