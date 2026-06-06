use lol_html::{HtmlRewriter, Settings, element};
use remyx::Application;
use remyx::element::{Element, container::Container, list::PickList};
use remyx::ratatui::crossterm;
use remyx::ratatui::crossterm::{
    event::{EnableBracketedPaste, EnableFocusChange, EnableMouseCapture},
    terminal::{EnterAlternateScreen, enable_raw_mode},
};
use remyx::ratatui::{
    Terminal,
    layout::{Constraint, Layout},
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, ListItem, Paragraph},
};
use remyx::runtime::tokio::Tokio;
use remyx::task::Task;
use std::io;

fn main() -> io::Result<()> {
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

    remyx::run::<App, Tokio, _>(terminal)
}

pub struct App {
    html_page: String,
}

#[derive(Copy, Clone)]
pub enum Link {
    Rust,
    C,
    Java,
}

impl Into<ListItem<'static>> for Link {
    fn into(self) -> ListItem<'static> {
        match self {
            Link::Rust => ListItem::new("Rust"),
            Link::C => ListItem::new("C"),
            Link::Java => ListItem::new("Java"),
        }
    }
}

pub enum Message {
    LinkChanged(Link),
    ContentChanged(String),
}

impl Application for App {
    type Message = Message;

    fn init() -> (Self, Option<Task<Message>>) {
        let self_ = Self {
            html_page: String::new(),
        };
        (self_, None)
    }

    fn view(&self) -> impl Element<Self::Message> {
        Container::layout(Layout::vertical(vec![
            Constraint::Percentage(20),
            Constraint::Percentage(80),
        ]))
        .with(
            PickList::new(vec![Link::C, Link::Rust, Link::Java])
                .block(
                    Block::default()
                        .title_top("Links")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .style(Style::default().fg(Color::White))
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::REVERSED | Modifier::BOLD),
                )
                .highlight_symbol("> ")
                .on_select(|item| Message::LinkChanged(*item)),
        )
        .with(
            Paragraph::new(self.html_page.to_string())
                .centered()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Html Page")
                        .border_style(Style::default().fg(Color::Blue)),
                )
                .style(Style::default().fg(Color::White)),
        )
    }

    fn update(&mut self, message: Self::Message) -> Option<Task<Self::Message>> {
        match message {
            Message::LinkChanged(link) => {
                let task = match link {
                    Link::C => {
                        html_page("https://es.wikipedia.org/wiki/C_(lenguaje_de_programación)")
                    }
                    Link::Rust => {
                        html_page("https://es.wikipedia.org/wiki/Rust_(lenguaje_de_programación)")
                    }
                    Link::Java => {
                        html_page("https://es.wikipedia.org/wiki/Java_(lenguaje_de_programación)")
                    }
                };
                Some(task)
            }
            Message::ContentChanged(content) => {
                self.html_page = content;
                None
            }
        }
    }
}

fn html_page(url: &'static str) -> Task<Message> {
    Task::new(async move {
        let client = reqwest::Client::builder()
            .user_agent("my-app/0.1 (https://example.com/contact)")
            .build()
            .unwrap();

        let response = client.get(url).send().await.unwrap();
        let html = response.text().await.unwrap();
        let mut output = Vec::new();
        let mut rewriter = HtmlRewriter::new(
            Settings {
                element_content_handlers: vec![
                    // Remove unwanted tags completely
                    element!("script", |el| {
                        el.remove();
                        Ok(())
                    }),
                    element!("style", |el| {
                        el.remove();
                        Ok(())
                    }),
                    element!("img", |el| {
                        el.remove();
                        Ok(())
                    }),
                ],
                ..Settings::default()
            },
            |c: &[u8]| output.extend_from_slice(c),
        );

        rewriter.write(html.as_bytes()).unwrap();
        rewriter.end().unwrap();

        let html = String::from_utf8(output)
            .unwrap()
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        Message::ContentChanged(html)
    })
}
