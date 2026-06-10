use futures::StreamExt;
use ratatui_core::{backend, terminal::Terminal};
use std::io;

use crate::{
    element::{Element, Tree},
    runtime::{self, JoinHandle},
    stream::{self},
    subscription, task,
    terminal::{self, EventResult},
};

pub struct Runner<'a, Application, Runtime, Backend>
where
    Application: crate::Application,
    Runtime: runtime::Runtime,
    Backend: backend::Backend,
{
    terminal: Terminal<Backend>,
    app: Application,
    tree: Tree,
    tasks: task::Pending<Runtime, Application::Message>,
    subscriptions: subscription::Active<Application::Message>,
    terminal_events: stream::Tee<Runtime, EventResult>,
    rt: &'a Runtime,
}

impl<'a, Application, Runtime, Backend> Runner<'a, Application, Runtime, Backend>
where
    Application: crate::Application,
    Runtime: runtime::Runtime,
    Backend: backend::Backend,
{
    pub fn new(terminal: Terminal<Backend>, rt: &'a Runtime) -> io::Result<Self> {
        let (app, task) = Application::init::<Runtime>();
        let tree = Tree::init(&app.view());
        let tasks = task::Pending::new();
        if let Some(task) = task {
            let task = rt.spawn(task).into_future();
            tasks.register(task);
        }
        let subscriptions = subscription::Active::new();

        Ok(Self {
            terminal,
            app,
            tree,
            tasks,
            subscriptions,
            terminal_events: terminal::events(),
            rt,
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
                    Some(Err(kind)) =>  {
                        return Err(io::Error::new(kind, kind.to_string()));
                    },
                    None => break,
                },
            }

            self.redraw();
            if self.app.exit() {
                break;
            }
        }

        Ok(())
    }

    fn update(&mut self, msg: Application::Message) {
        if let Some(task) = self.app.update::<Runtime>(msg) {
            let task = self.rt.spawn(task).into_future();
            self.tasks.register(task);
        }
    }

    fn handle_terminal_event(&mut self, event: crossterm::event::Event) -> bool {
        let area = self.terminal.get_frame().area();
        let mut shell = Shell::new();
        self.app.view().update(&self.tree, area, event, &mut shell);

        if !shell.should_redraw() {
            return false;
        }

        for msg in shell.clear() {
            self.update(msg);
        }

        true
    }

    fn redraw(&mut self) {
        let view = self.app.view();
        self.tree.diff(&view);
        self.subscriptions.diff(
            &mut self.terminal_events,
            self.app.subscription::<Runtime>(),
        );

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
