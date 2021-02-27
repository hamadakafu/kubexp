use std::{error::Error, io};

use crate::event::{Event, Events};
use anyhow::{format_err, Context, Result};
use termion::raw::RawTerminal;
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::layout::Rect;
use tui::{backend::Backend, Frame};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use unicode_width::UnicodeWidthStr;

enum InputMode {
    Normal,
    Editing,
}

use crate::event;

/// App holds the state of the application
pub struct App {
    /// Current value of the input box
    input: String,
    /// kubectl explainの結果
    output: String,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,
    events: Events,
}

impl Default for App {
    fn default() -> App {
        // Setup event handlers
        let mut events = Events::new();

        App {
            input: String::new(),
            input_mode: InputMode::Editing,
            output: String::new(),
            messages: Vec::new(),
            events,
        }
    }
}

impl App {
    pub fn initialize(&mut self) -> Result<()> {
        let output = std::process::Command::new("kubectl")
            .arg("api-resources")
            .arg("-o")
            .arg("name")
            .output()
            .context(format!("kubectl explain {}", self.input))?;
        self.output = match String::from_utf8(output.stdout) {
            Ok(value) => value,
            Err(error) => format_err!("{:?}", error).to_string(),
        };
        return Ok(());
    }
    pub fn draw_help_message<B>(&self, f: &mut Frame<B>, chunks: &Vec<Rect>)
    where
        B: Backend,
    {
        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    Span::raw("Press "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to exit, "),
                    Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to start editing."),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Editing => (
                vec![
                    Span::raw("Press "),
                    Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to stop editing, "),
                    Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to record the message"),
                ],
                Style::default(),
            ),
        };

        let mut text = Text::from(Spans::from(msg));
        text.patch_style(style);
        let help_message = Paragraph::new(text);
        f.render_widget(help_message, chunks[0]);
    }

    pub fn draw_input<B>(&self, f: &mut Frame<B>, chunks: &Vec<Rect>)
    where
        B: Backend,
    {
        let input = Paragraph::new(self.input.as_ref())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::default().borders(Borders::ALL).title("Input"));
        f.render_widget(input, chunks[1]);
        f.set_cursor(
            // Put cursor past the end of the input text
            chunks[1].x + self.input.width() as u16 + 1,
            // Move one line down, from the border to the input line
            chunks[1].y + 1,
        );
    }

    pub fn draw_result<B>(&self, f: &mut Frame<B>, chunks: &Vec<Rect>)
    where
        B: Backend,
    {
        let explain = Paragraph::new(self.output.clone())
            .style(Style::default())
            .block(Block::default().borders(Borders::ALL).title("kubexp"));

        f.render_widget(explain, chunks[2]);
    }

    pub fn handle_input(&mut self) -> Result<()> {
        // Handle input
        if let Event::Input(input) = self.events.next()? {
            match self.input_mode {
                InputMode::Normal => match input {
                    Key::Char('i') => {
                        self.input_mode = InputMode::Editing;
                        self.events.disable_exit_key();
                    }
                    Key::Char('q') => {
                        std::process::exit(0);
                    }
                    _ => {}
                },
                InputMode::Editing => match input {
                    Key::Char('\n') => {
                        let output = std::process::Command::new("kubectl")
                            .arg("explain")
                            .arg(&self.input)
                            .output()
                            .context(format!("kubectl explain {}", self.input));
                        self.output = match output {
                            Ok(output) => match String::from_utf8(output.stdout) {
                                Ok(value) => value,
                                Err(error) => format_err!("{:?}", error).to_string(),
                            },
                            Err(error) => format_err!("{:?}", error).to_string(),
                        };
                    }
                    Key::Char(c) => {
                        self.input.push(c);
                    }
                    Key::Backspace => {
                        self.input.pop();
                    }
                    Key::Esc => {
                        self.input_mode = InputMode::Normal;
                        self.events.enable_exit_key();
                    }
                    _ => {}
                },
            }
        }
        return Ok(());
    }
}
