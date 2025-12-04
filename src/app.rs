use crate::nix::{ConfigOption, ConfigSource};
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::Widget,
};
use std::{
    io::Result,
    sync::mpsc::{self, Receiver},
};

pub struct App {
    pub exit: bool,
    pub system_modules: Vec<ConfigOption>,
    pub home_modules: Vec<ConfigOption>,
    pub current_file: ConfigSource,
    pub selected_index: usize,
    pub last_action_status: String,
}

pub enum Event {
    Input(KeyEvent),
    EditFile,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal, rx: Receiver<Event>) -> Result<()> {
        while !self.exit {
            let event: Event = match rx.recv() {
                Ok(ev) => ev,
                Err(_) => break,
            };
            match event {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
                // Event::EditFile => self.handle_file_edit_events()?,
                _ => {}
            }
            terminal.draw(|frame: &mut Frame<'_>| self.draw(frame))?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame<'_>) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        // q - wyjście z programu
        if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char('q') {
            self.exit = true;
        }
        // ↑ - scroll w górę
        else if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Up {
            self.selected_index = self.selected_index.saturating_sub(1);
        }
        // ↓ - scroll w dół
        else if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Down {
            self.selected_index = self.selected_index.saturating_add(1);
        }
        // ENTER - zmień wartość boolen danej opcji
        else if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Enter {
        }
        // 1 - zmień plik na ConfigSource::System (pomiń wykonywaniej, jeżeli już jest wybrany)
        else if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char('1') {
            if self.current_file != ConfigSource::System {
                self.current_file = ConfigSource::System;
            } else {
                self.current_file = ConfigSource::Home;
            }
        }
        // 2 - zmień plik na ConfigSource::Home (pomiń wykonywaniej, jeżeli już jest wybrany)
        else if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char('2') {
            if self.current_file != ConfigSource::Home {
                self.current_file = ConfigSource::Home;
            } else {
                self.current_file = ConfigSource::System;
            }
        }

        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let vertical_layout: Layout = Layout::vertical([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ]);

        let [title_area, content_area, footer_area] = vertical_layout.areas(area);

        let (system_file, home_file) = match self.current_file {
            ConfigSource::System => ("(System)".blue().bold(), "Home".reset()),
            ConfigSource::Home => ("System".reset(), "(Home)".blue().bold()),
        };
        let navbar: Line<'_> = Line::from(vec![
            Span::from("GarandOS TUI".bold().italic().yellow()),
            Span::from(" * ".reset()),
            system_file,
            Span::from(" [1]".reset()),
            Span::from(" * ".reset()),
            home_file,
            Span::from(" [2]".reset()),
            Span::from(" *".reset()),
            Span::from(" Quit".reset()),
            Span::from(" [q]".reset()),
        ])
        .left_aligned();

        navbar.render(title_area, buf);
    }
}

pub fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        match event::read() {
            Ok(ev) => {
                if let event::Event::Key(key_event) = ev {
                    if tx.send(Event::Input(key_event)).is_err() {
                        break;
                    }
                }
            }
            Err(_) => {
                continue;
            }
        }
    }
}
