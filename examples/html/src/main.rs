use lol_html::{HtmlRewriter, Settings, element};
use remyx::crossterm::crossterm::event::{
    DisableMouseCapture, EnableBracketedPaste, EnableFocusChange, EnableMouseCapture, KeyCode,
};
use remyx::crossterm::crossterm::execute;
use remyx::crossterm::crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use remyx::element::{Element, container::Container};
use remyx::ratatui::layout::{Constraint, Layout};
use remyx::ratatui::style::{Color, Modifier, Style};
use remyx::runtime::tokio::Tokio;
use remyx::subscription::Subscription;
use remyx::task::Task;
use remyx::terminal::crossterm::Crossterm;
use remyx::widgets::block::Block;
use remyx::widgets::borders::Borders;
use remyx::widgets::list::{List, ListItem};
use remyx::widgets::paragraph::Paragraph;
use remyx::{Application, runtime};
use std::io;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(
        io::stdout(),
        EnableMouseCapture,
        EnableFocusChange,
        EnableBracketedPaste,
    )?;

    let terminal = Crossterm::<Tokio>::new();
    remyx::run::<App, Tokio, _>(terminal)?;

    execute!(io::stdout(), DisableMouseCapture)?;
    disable_raw_mode()
}

pub struct App {
    html_page: String,
    exit: bool,
}

pub enum Message {
    LinkChanged(Link),
    ContentChanged(String),
    Exit,
}

impl Application for App {
    type Message = Message;

    fn init<Runtime: runtime::Runtime>() -> (Self, Option<Task<Message>>) {
        let self_ = Self {
            html_page: String::new(),
            exit: false,
        };
        (self_, None)
    }

    fn view(&self) -> impl Element<Self::Message> {
        Container::layout(Layout::vertical([
            Constraint::Percentage(20),
            Constraint::Percentage(80),
        ]))
        .with(
            List::new([Link::C, Link::Java, Link::Rust])
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

    fn update<Runtime: runtime::Runtime>(
        &mut self,
        message: Self::Message,
    ) -> Option<Task<Self::Message>> {
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
            Message::Exit => {
                self.exit = true;
                None
            }
        }
    }

    fn subscription<Terminal: remyx::terminal::Terminal>(
        &self,
    ) -> Vec<Subscription<Terminal, Self::Message>> {
        let exit = Subscription::key(|key| {
            if key.code.eq(&KeyCode::Esc) {
                Some(Message::Exit)
            } else {
                None
            }
        });

        vec![exit]
    }

    fn exit(&self) -> bool {
        self.exit
    }
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
