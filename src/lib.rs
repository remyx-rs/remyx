use crate::subscription::Subscription;

use self::{element::Element, runner::Runner, task::Task};
use ratatui::{Terminal, backend};
use std::io;

pub use ratatui;

pub mod element;
mod runner;
pub mod runtime;
pub mod stream;
pub mod subscription;
pub mod task;
mod terminal;

pub fn run<Application, Runtime, Backend>(terminal: Terminal<Backend>) -> io::Result<()>
where
    Runtime: runtime::Runtime,
    Application: self::Application,
    Backend: backend::Backend,
{
    let rt = Runtime::new(0);
    let runner = Runner::<Application, Runtime, Backend>::new(terminal, &rt)?;
    rt.block_on(runner.run())
}

pub fn run_with<Application, Runtime, Backend>(
    threads: usize,
    terminal: Terminal<Backend>,
) -> io::Result<()>
where
    Runtime: runtime::Runtime,
    Application: self::Application,
    Backend: backend::Backend,
{
    let rt = Runtime::new(0);
    let bg_rt = Runtime::new(threads.max(1));
    let runner = Runner::<Application, Runtime, Backend>::new(terminal, &bg_rt)?;
    rt.block_on(runner.run())
}

pub trait Application {
    type Message: Send + 'static;

    fn init<Runtime: runtime::Runtime>() -> (Self, Option<Task<Self::Message>>)
    where
        Self: Sized;
    fn view(&self) -> impl Element<Self::Message>;
    fn update<Runtime: runtime::Runtime>(
        &mut self,
        message: Self::Message,
    ) -> Option<Task<Self::Message>>;

    fn subscription<Runtime: runtime::Runtime>(&self) -> Vec<Subscription<Runtime, Self::Message>> {
        vec![]
    }

    fn exit(&self) -> bool {
        false
    }
}
