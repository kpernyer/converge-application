// Copyright 2024-2025 Aprio One AB, Sweden
// Author: Kenneth Pernyer, kenneth@aprio.one
// SPDX-License-Identifier: MIT
// See LICENSE file in the project root for full license information.

//! TUI (Terminal User Interface) for Converge.
//!
//! Provides an interactive terminal interface for:
//! - Submitting and monitoring convergence jobs
//! - Visualizing context state and agent execution
//! - Reviewing proposals and providing human-in-the-loop feedback

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};
use std::io::{self, stdout};
use thiserror::Error;

/// TUI-specific errors.
#[derive(Debug, Error)]
pub enum TuiError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Terminal setup failed: {0}")]
    TerminalSetup(String),
}

/// TUI application state.
pub struct TuiApp {
    /// Whether the app should quit.
    should_quit: bool,
    /// Current status message.
    status: String,
}

impl TuiApp {
    /// Create a new TUI application.
    pub fn new() -> Self {
        Self {
            should_quit: false,
            status: "Welcome to Converge".to_string(),
        }
    }

    /// Run the TUI application.
    pub fn run(&mut self) -> Result<(), TuiError> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Main loop
        let result = self.main_loop(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn main_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), TuiError> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                self.should_quit = true;
                            }
                            KeyCode::Char('j') => {
                                self.status = "Submit Job (not implemented)".to_string();
                            }
                            KeyCode::Char('s') => {
                                self.status = "View Status (not implemented)".to_string();
                            }
                            KeyCode::Char('h') => {
                                self.status = "Help: q=quit, j=job, s=status".to_string();
                            }
                            _ => {}
                        }
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(10),    // Main content
                Constraint::Length(3),  // Status bar
            ])
            .split(area);

        // Header
        let header = Paragraph::new("Converge TUI")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, chunks[0]);

        // Main content
        let content = Paragraph::new(vec![
            Line::from(""),
            Line::from("  Press 'j' to submit a job"),
            Line::from("  Press 's' to view status"),
            Line::from("  Press 'h' for help"),
            Line::from("  Press 'q' or Esc to quit"),
            Line::from(""),
            Line::from("  (TUI implementation in progress)"),
        ])
        .block(Block::default().borders(Borders::ALL).title("Main"));
        frame.render_widget(content, chunks[1]);

        // Status bar
        let status = Paragraph::new(self.status.as_str())
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title("Status"));
        frame.render_widget(status, chunks[2]);
    }
}

impl Default for TuiApp {
    fn default() -> Self {
        Self::new()
    }
}
