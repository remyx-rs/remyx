use remyx::crossterm::crossterm::event::{
    DisableMouseCapture, EnableBracketedPaste, EnableFocusChange, EnableMouseCapture, KeyCode,
    KeyModifiers,
};
use remyx::crossterm::crossterm::execute;
use remyx::crossterm::crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use remyx::element::container::Container;
use remyx::ratatui::layout::{Constraint, Layout};
use remyx::ratatui::style::{Color, Modifier, Style};
use remyx::runtime::tokio::Tokio;
use remyx::terminal::crossterm::Crossterm;
use remyx::widgets::block::Block;
use remyx::widgets::borders::Borders;
use remyx::widgets::paragraph::Paragraph;
use remyx::widgets::table::{Row, Table};
use remyx::{Application, Element, Subscription, Task, runtime, terminal};
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
    detail: String,
    exit: bool,
}

pub enum Message {
    DetailChanged(String),
    Exit,
}

impl Application for App {
    type Message = Message;

    fn init<Runtime: runtime::Runtime>() -> (Self, Option<Task<Message>>) {
        let self_ = Self {
            detail: String::new(),
            exit: false,
        };
        (self_, None)
    }

    fn view(&self) -> impl Element<Self::Message> {
        Container::layout(Layout::vertical([
            Constraint::Percentage(70),
            Constraint::Percentage(30),
        ]))
        .with(
            Table::new(
                Language::ALL,
                [
                    Constraint::Percentage(40),
                    Constraint::Percentage(20),
                    Constraint::Percentage(40),
                ],
            )
            .header(
                Row::new(vec!["Language", "Year", "Designer / Notes"]).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            )
            .block(
                Block::default()
                    .title_top("Languages")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::White))
            .row_highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::REVERSED | Modifier::BOLD),
            )
            .highlight_symbol("> ")
            .on_select(|lang| {
                Message::DetailChanged(format!(
                    "{} was created in {} by {}.",
                    lang.name(),
                    lang.year(),
                    lang.designer()
                ))
            }),
        )
        .with(
            Paragraph::new(self.detail.clone())
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

    fn update<Runtime: runtime::Runtime>(
        &mut self,
        message: Self::Message,
    ) -> Option<Task<Self::Message>> {
        match message {
            Message::DetailChanged(detail) => {
                self.detail = detail;
                None
            }
            Message::Exit => {
                self.exit = true;
                None
            }
        }
    }

    fn subscription<Terminal: terminal::Terminal>(
        &self,
    ) -> Vec<Subscription<Terminal, Self::Message>> {
        let exit = Subscription::key(|key| {
            (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                .then_some(Message::Exit)
        });

        vec![exit]
    }

    fn exit(&self) -> bool {
        self.exit
    }
}

#[derive(Copy, Clone)]
pub enum Language {
    Rust,
    C,
    Cpp,
    Java,
    Python,
    Go,
    JavaScript,
    TypeScript,
    CSharp,
    Ruby,
    Swift,
    Kotlin,
    Haskell,
    Scala,
    Elixir,
    Zig,
    Lua,
    Perl,
}

impl Language {
    const ALL: [Language; 18] = [
        Language::Rust,
        Language::C,
        Language::Cpp,
        Language::Java,
        Language::Python,
        Language::Go,
        Language::JavaScript,
        Language::TypeScript,
        Language::CSharp,
        Language::Ruby,
        Language::Swift,
        Language::Kotlin,
        Language::Haskell,
        Language::Scala,
        Language::Elixir,
        Language::Zig,
        Language::Lua,
        Language::Perl,
    ];

    fn name(self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Java => "Java",
            Language::Python => "Python",
            Language::Go => "Go",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::CSharp => "C#",
            Language::Ruby => "Ruby",
            Language::Swift => "Swift",
            Language::Kotlin => "Kotlin",
            Language::Haskell => "Haskell",
            Language::Scala => "Scala",
            Language::Elixir => "Elixir",
            Language::Zig => "Zig",
            Language::Lua => "Lua",
            Language::Perl => "Perl",
        }
    }

    fn year(self) -> u16 {
        match self {
            Language::Rust => 2010,
            Language::C => 1972,
            Language::Cpp => 1985,
            Language::Java => 1995,
            Language::Python => 1991,
            Language::Go => 2009,
            Language::JavaScript => 1995,
            Language::TypeScript => 2012,
            Language::CSharp => 2000,
            Language::Ruby => 1995,
            Language::Swift => 2014,
            Language::Kotlin => 2011,
            Language::Haskell => 1990,
            Language::Scala => 2004,
            Language::Elixir => 2011,
            Language::Zig => 2016,
            Language::Lua => 1993,
            Language::Perl => 1987,
        }
    }

    fn designer(self) -> &'static str {
        match self {
            Language::Rust => "Graydon Hoare",
            Language::C => "Dennis Ritchie",
            Language::Cpp => "Bjarne Stroustrup",
            Language::Java => "James Gosling",
            Language::Python => "Guido van Rossum",
            Language::Go => "Robert Griesemer",
            Language::JavaScript => "Brendan Eich",
            Language::TypeScript => "Anders Hejlsberg",
            Language::CSharp => "Anders Hejlsberg",
            Language::Ruby => "Yukihiro Matsumoto",
            Language::Swift => "Chris Lattner",
            Language::Kotlin => "JetBrains",
            Language::Haskell => "FP Committee",
            Language::Scala => "Martin Odersky",
            Language::Elixir => "José Valim",
            Language::Zig => "Andrew Kelley",
            Language::Lua => "Roberto Ierusalimschy",
            Language::Perl => "Larry Wall",
        }
    }

    /// Extra description lines shown below the designer. The varying number of lines gives each
    /// row a different height, which is what exercises the variable-height click hit-testing.
    fn notes(self) -> &'static [&'static str] {
        match self {
            Language::Rust => &["Memory-safe systems", "Zero-cost abstractions"],
            Language::C => &[],
            Language::Cpp => &["Multi-paradigm"],
            Language::Java => &["Runs on the JVM", "Write once, run anywhere"],
            Language::Python => &["Batteries included"],
            Language::Go => &[],
            Language::JavaScript => &["The language of the web"],
            Language::TypeScript => &["Typed superset of JS", "Compiles to JavaScript"],
            Language::CSharp => &["Built for .NET"],
            Language::Ruby => &[],
            Language::Swift => &["Apple platforms", "Safe and fast"],
            Language::Kotlin => &["Modern JVM language"],
            Language::Haskell => &[],
            Language::Scala => &["Fuses OOP and FP"],
            Language::Elixir => &["Built on the BEAM", "Highly concurrent"],
            Language::Zig => &["A better C"],
            Language::Lua => &[],
            Language::Perl => &["Text processing"],
        }
    }
}

impl From<Language> for Row<'static> {
    fn from(language: Language) -> Self {
        let mut designer_lines = vec![language.designer().to_string()];
        designer_lines.extend(language.notes().iter().map(|note| note.to_string()));

        // Height grows with the number of description lines, so rows have varying sizes. The
        // bottom margin adds a blank separator line that the hit-testing must also account for.
        let height = designer_lines.len() as u16;

        Row::new(vec![
            language.name().to_string(),
            language.year().to_string(),
            designer_lines.join("\n"),
        ])
        .height(height)
        .bottom_margin(1)
    }
}
