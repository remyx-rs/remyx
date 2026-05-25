use std::io;

use crossterm::{
    event::{EnableBracketedPaste, EnableFocusChange, EnableMouseCapture},
    terminal::{EnterAlternateScreen, enable_raw_mode},
};
use ratatui::{
    Terminal,
    prelude::CrosstermBackend,
    style::{Color, Modifier},
    widgets::List,
};
use remyx::{Application, element::Element};

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();

    crossterm::execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableFocusChange,
        EnableBracketedPaste,
    )?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    remyx::run::<App, _>(terminal).await
}

pub struct App {}

impl Application for App {
    type Message = ();

    fn init() -> Self {
        Self {}
    }

    fn view(&self) -> impl Element<Self::Message> + use<> {
        let items = ["Item 1", "Item 2", "Item 3", "Item 4"];
        List::new(items)
            .style(Color::White)
            .highlight_style(Modifier::REVERSED)
            .highlight_symbol("> ")
    }

    fn update(&mut self, message: Self::Message) {}
}
