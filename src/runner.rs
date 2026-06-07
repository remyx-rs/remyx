use futures::StreamExt;
use ratatui::{Terminal, backend};
use std::{io, marker::PhantomData};

use crate::{
    element::{Element, Tree},
    runtime::{self, JoinHandle},
    stream::LocalBoxFusedStream,
    subscription, task, terminal,
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
    shell: Shell<Application::Message>,
    tasks: task::Pending<Runtime, Application::Message>,
    subscriptions: subscription::Set<Application::Message>,
    terminal_events: LocalBoxFusedStream<io::Result<crossterm::event::Event>>,
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
        let shell = Shell::new();
        let tasks = task::Pending::new();
        if let Some(task) = task {
            let task = Runtime::spawn(task).into_future();
            tasks.register(task);
        }
        let subscriptions = subscription::Set::new();

        Ok(Self {
            terminal,
            app,
            tree,
            shell,
            tasks,
            subscriptions,
            terminal_events: terminal::events(),
            _rt_marker: PhantomData,
        })
    }

    pub async fn run(mut self) -> io::Result<()> {
        self.redraw();
        loop {
            futures::select_biased! {
                subscription = self.subscriptions.next() => if let Some(msg) = subscription {
                    self.update(msg);
                },
                task = self.tasks.next() => match task {
                    Some(Ok(msg)) => {
                        self.update(msg);
                    }
                    Some(Err(_)) => {

                    }
                    None => {

                    }
                },
                event = self.terminal_events.next() => match event {
                    Some(Ok(event)) => {
                        if !self.handle_terminal_event(event) {
                            continue;
                        }
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
            let task = Runtime::spawn(task).into_future();
            self.tasks.register(task);
        }
    }

    fn handle_terminal_event(&mut self, event: crossterm::event::Event) -> bool {
        let area = self.terminal.get_frame().area();
        self.app
            .view()
            .update(&self.tree, area, event, &mut self.shell);

        if !self.shell.should_redraw() {
            return false;
        }

        for msg in self.shell.clear() {
            self.update(msg);
        }

        true
    }

    fn redraw(&mut self) {
        let view = self.app.view();
        self.tree.diff(&view);
        self.subscriptions.diff(self.app.subscription());

        let _ = self.terminal.draw(|frame| {
            view.draw(&self.tree, frame.area(), frame.buffer_mut());
        });
    }
}

#[derive(Debug)]
pub struct Shell<Message> {
    messages: Vec<Message>,
    redraw_requested: bool,
}

impl<Message> Shell<Message> {
    fn new() -> Self {
        Self {
            messages: Vec::new(),
            redraw_requested: false,
        }
    }

    pub fn publish(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn should_redraw(&self) -> bool {
        self.redraw_requested || !self.messages.is_empty()
    }

    pub fn request_redraw(&mut self) {
        self.redraw_requested = true;
    }

    pub fn clear(&mut self) -> Vec<Message> {
        let messages = self.messages.drain(..).collect::<Vec<_>>();
        self.redraw_requested = false;
        messages
    }
}
