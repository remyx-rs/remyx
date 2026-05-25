use std::io;

use crossterm::{
    event::{EnableBracketedPaste, EnableFocusChange, EnableMouseCapture},
    terminal::{EnterAlternateScreen, enable_raw_mode},
};
use ratatui::{
    Terminal,
    layout::{Constraint, Layout},
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, ListItem, Paragraph},
};
use remyx::{
    Application,
    element::{Element, container::Container, list::PickList},
};

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

pub struct App {
    detail: &'static str,
}

#[derive(Copy, Clone)]
pub enum Section {
    Rust,
    CloudComputing,
}

impl Into<ListItem<'static>> for Section {
    fn into(self) -> ListItem<'static> {
        match self {
            Section::Rust => ListItem::new("Rust"),
            Section::CloudComputing => ListItem::new("Cloud Computing"),
        }
    }
}

pub enum Message {
    SectionChanged(Section),
}

impl Application for App {
    type Message = Message;

    fn init() -> Self {
        Self { detail: "" }
    }

    fn view(&self) -> impl Element<Self::Message> + use<> {
        Container::layout(Layout::vertical(vec![
            Constraint::Percentage(10),
            Constraint::Percentage(90),
        ]))
        .with(
            PickList::list(vec![Section::Rust, Section::CloudComputing], |list| {
                list.block(
                    Block::default()
                        .borders(Borders::BOTTOM)
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .style(Style::default().fg(Color::White))
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::REVERSED | Modifier::BOLD),
                )
                .highlight_symbol("> ")
            })
            .on_select(|item| Some(Message::SectionChanged(*item))),
        )
        .with(
            Paragraph::new(self.detail)
                .centered()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Details")
                        .border_style(Style::default().fg(Color::Blue)),
                )
                .style(Style::default().fg(Color::White)),
        )
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::SectionChanged(section) => {
                self.detail = match section {
                    Section::Rust => {
                        r#"
Rust is a systems programming language focused on safety, speed, and concurrency.
It achieves memory safety without using a garbage collector by enforcing strict
ownership and borrowing rules at compile time. Developers often choose Rust for
performance-critical applications such as operating systems, game engines, web
servers, and embedded devices.

One of Rust’s strongest features is its modern tooling ecosystem. Cargo, the
built-in package manager and build system, simplifies dependency management,
testing, formatting, and documentation generation. Combined with the compiler’s
helpful error messages, Rust provides a productive development experience despite
its steep learning curve.

The language also promotes fearless concurrency. Threads and asynchronous tasks
can be implemented with reduced risk of data races because the compiler enforces
safe access patterns. This makes Rust attractive for applications that require
high reliability and scalability under heavy workloads.
"#
                    }
                    Section::CloudComputing => {
                        r#"
The growth of cloud computing has transformed how organizations deploy and scale
software services. Instead of relying on physical infrastructure alone,
businesses can now use distributed platforms to provision computing resources
on demand. This flexibility reduces operational costs and allows engineering
teams to focus more on product development rather than hardware maintenance.

Modern cloud platforms provide services for storage, networking, machine
learning, and real-time analytics. Companies can quickly launch applications
that automatically scale during periods of high traffic and reduce resource
usage during quieter times. This dynamic allocation improves efficiency and
supports rapid experimentation.

Security and monitoring remain critical concerns in cloud environments.
Organizations must implement strong authentication systems, encryption policies,
and continuous observability practices. As cloud adoption continues to expand,
engineers are investing heavily in automation and infrastructure-as-code tools
to maintain reliable and reproducible deployments across multiple regions.
"#
                    }
                }
            }
        }
    }
}
