use self::{element::Element, runner::Runner, task::Task};
use crate::subscription::Subscription;
use std::io;

pub use ratatui_core as ratatui;
pub use ratatui_crossterm as crossterm;
pub use remyx_widgets as widgets;

pub mod element;
mod runner;
pub mod runtime;
pub mod stream;
pub mod subscription;
pub mod task;
pub mod terminal;

pub fn run<Application, Runtime, Terminal>(terminal: Terminal) -> io::Result<()>
where
    Runtime: runtime::Runtime,
    Application: self::Application,
    Terminal: self::terminal::Terminal,
{
    let rt = Runtime::new(0);
    let runner = Runner::<Application, Runtime, Terminal>::new(terminal, &rt)?;
    rt.block_on(runner.run())
}

pub fn run_with<Application, Runtime, Terminal>(
    threads: usize,
    terminal: Terminal,
) -> io::Result<()>
where
    Runtime: runtime::Runtime,
    Application: self::Application,
    Terminal: self::terminal::Terminal,
{
    let rt = Runtime::new(0);
    let bg_rt = Runtime::new(threads.max(1));
    let runner = Runner::<Application, Runtime, Terminal>::new(terminal, &bg_rt)?;
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

    fn subscription<Terminal: terminal::Terminal>(
        &self,
    ) -> Vec<Subscription<Terminal, Self::Message>> {
        vec![]
    }

    fn exit(&self) -> bool {
        false
    }
}
