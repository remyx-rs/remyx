use self::{element::Element, runner::Runner, stream::Source, task::Task};
use ratatui::{Terminal, backend};
use std::io;

pub use ratatui;

pub mod element;
mod runner;
pub mod runtime;
pub mod stream;
pub mod task;
mod terminal;

pub fn run<Application, Runtime, Backend>(terminal: Terminal<Backend>) -> io::Result<()>
where
    Runtime: runtime::Runtime,
    Application: self::Application,
    Backend: backend::Backend,
{
    let rt = Runtime::new(1);
    with_runtime::<Application, Runtime, Backend>(rt, terminal)
}

pub fn with_runtime<Application, Runtime, Backend>(
    runtime: Runtime,
    terminal: Terminal<Backend>,
) -> io::Result<()>
where
    Runtime: runtime::Runtime,
    Application: self::Application,
    Backend: backend::Backend,
{
    let runner = Runner::<Application, Runtime, Backend>::new(terminal)?;
    runtime.block_on(runner.run())
}

pub trait Application {
    type Message: Send + 'static;

    fn init() -> (Self, Option<Task<Self::Message>>)
    where
        Self: Sized;
    fn view(&self) -> impl Element<Self::Message>;
    fn update(&mut self, message: Self::Message) -> Option<Task<Self::Message>>;
    fn subscription(&self) -> Vec<Source<Self::Message>> {
        vec![]
    }
}
