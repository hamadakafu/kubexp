#![allow(dead_code, unused_imports, unreachable_code)]
mod app;
mod event;

use crate::event::{Event, Events};
use std::{error::Error, io};
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
use anyhow::{Context, Result};

enum InputMode {
    Normal,
    Editing,
}

use app::App;

fn main() -> Result<()> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create default app state
    let mut app = App::default();
    app.initialize()?;
    terminal.clear().context("wtf")?;
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                // terminalを3つに分割
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(3),
                        Constraint::Min(1),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            app.draw_help_message(f, &chunks);
            app.draw_input(f, &chunks);
            app.draw_result(f, &chunks);
        })?;

        app.handle_input()?;
    }
    Ok(())
}

