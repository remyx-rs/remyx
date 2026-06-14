use futures::StreamExt;
use std::{io, mem};

use crate::{
    element::{Element, Tree},
    runtime::{self, JoinHandle},
    subscription, task,
    terminal::{self, Cursor},
};

pub struct Runner<'a, Application, Runtime, Terminal>
where
    Application: crate::Application,
    Runtime: runtime::Runtime,
    Terminal: terminal::Terminal,
{
    terminal: Terminal,
    app: Application,
    tree: Tree,
    tasks: task::Pending<Runtime, Application::Message>,
    subscriptions: subscription::Active<Application::Message>,
    rt: &'a Runtime,
}

impl<'a, Application, Runtime, Terminal> Runner<'a, Application, Runtime, Terminal>
where
    Application: crate::Application,
    Runtime: runtime::Runtime,
    Terminal: terminal::Terminal,
{
    pub fn new(terminal: Terminal, rt: &'a Runtime) -> io::Result<Self> {
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
                task = self.tasks.next() => if let Some(Ok(msg)) = task {
                    self.update(msg);
                },
                event = self.terminal.next() => match event {
                    Some(Ok(event)) => {
                        if !self.handle_terminal_event(event) {
                            continue;
                        }
                    }
                    Some(Err(kind)) =>  {
                        return Err(io::Error::from(kind));
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
        let cursor = self.terminal.mouse();
        let mut ctx = Context::new(cursor);

        self.app.view().update(&self.tree, area, event, &mut ctx);

        if !ctx.should_redraw() {
            return false;
        }

        for msg in ctx.messages() {
            self.update(msg);
        }

        true
    }

    fn redraw(&mut self) {
        let view = self.app.view();
        self.tree.diff(&view);
        self.subscriptions
            .diff(&mut self.terminal, self.app.subscription::<Terminal>());

        let _ = self.terminal.draw(|frame| {
            view.draw(&self.tree, frame.area(), frame.buffer_mut());
        });
    }
}

#[derive(Debug)]
pub struct Context<Message> {
    cursor: Cursor,
    messages: Vec<Message>,
    redraw: bool,
}

impl<Message> Context<Message> {
    fn new(cursor: Cursor) -> Self {
        Self {
            cursor,
            messages: Vec::new(),
            redraw: false,
        }
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn publish(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn should_redraw(&self) -> bool {
        self.redraw || !self.messages.is_empty()
    }

    pub fn redraw(&mut self) {
        self.redraw = true;
    }

    pub fn messages(&mut self) -> Vec<Message> {
        mem::take(&mut self.messages)
    }
}
