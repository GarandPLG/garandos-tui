use crate::nix::{
    ConfigOption, ConfigSource, collect_nix_options, get_nix_value_by_path, toggle_bool_at_path,
};
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use rnix::{Parse, Root, SyntaxNode};
use std::{
    fs,
    io::Result,
    path::PathBuf,
    sync::mpsc::{self, Receiver},
};

pub struct App {
    pub exit: bool,
    pub system_path: PathBuf,
    pub home_path: PathBuf,
    pub system_ast: Parse<Root>,
    pub home_ast: Parse<Root>,
    pub system_modules: Vec<ConfigOption>,
    pub home_modules: Vec<ConfigOption>,
    pub current_file: ConfigSource,
    pub selected_index: usize,
    pub last_action_status: String,
}

pub enum Event {
    Input(KeyEvent),
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal, rx: Receiver<Event>) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame: &mut Frame<'_>| self.draw(frame))?;

            let event: Event = match rx.recv() {
                Ok(ev) => ev,
                Err(_) => break,
            };
            match event {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
            }
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
        // 1/← - zmień plik na ConfigSource::System (pomiń wykonywaniej, jeżeli już jest wybrany)
        else if key_event.kind == KeyEventKind::Press
            && (key_event.code == KeyCode::Char('1') || key_event.code == KeyCode::Left)
        {
            self.current_file = ConfigSource::System;
            self.selected_index = 0_usize;
        }
        // 2/→ - zmień plik na ConfigSource::Home (pomiń wykonywaniej, jeżeli już jest wybrany)
        else if key_event.kind == KeyEventKind::Press
            && (key_event.code == KeyCode::Char('2') || key_event.code == KeyCode::Right)
        {
            self.current_file = ConfigSource::Home;
            self.selected_index = 0_usize;
        }
        // ↑ - scroll w górę
        else if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Up {
            self.selected_index = self.selected_index.saturating_sub(1);
        }
        // ↓ - scroll w dół
        else if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Down {
            let max_index = match self.current_file {
                ConfigSource::System => self.system_modules.len() - 1,
                ConfigSource::Home => self.home_modules.len() - 1,
            };
            self.selected_index = self.selected_index.saturating_add(1).min(max_index);
        }
        // ENTER/SPACE - zmień wartość boolen danej opcji
        else if key_event.kind == KeyEventKind::Press
            && (key_event.code == KeyCode::Enter || key_event.code == KeyCode::Char(' '))
        {
            let (modules, new_node, new_ast, new_modules, new_module_value) =
                self.get_selected_option_context();

            self.last_action_status = if let Some(module) = modules.get(self.selected_index) {
                match self.current_file {
                    ConfigSource::System => {
                        self.system_modules = new_modules;
                        self.system_ast = new_ast;
                        fs::write(&self.system_path, new_node.to_string())
                            .expect("Nie można zapisać pliku modułów systemowych");
                    }
                    ConfigSource::Home => {
                        self.home_modules = new_modules;
                        self.home_ast = new_ast;
                        fs::write(&self.home_path, new_node.to_string())
                            .expect("Nie można zapisać pliku modułów domowych");
                    }
                };
                format!(
                    "{} = {} -> {}",
                    module.path.clone(),
                    module.value.clone(),
                    new_module_value
                )
            } else {
                "Nothing changed".to_string()
            };
        }

        Ok(())
    }

    fn get_selected_option_context(
        &self,
    ) -> (
        Vec<ConfigOption>,
        SyntaxNode,
        Parse<Root>,
        Vec<ConfigOption>,
        bool,
    ) {
        let modules: &Vec<ConfigOption> = match self.current_file {
            ConfigSource::System => &self.system_modules,
            ConfigSource::Home => &self.home_modules,
        };
        let node: &SyntaxNode = match self.current_file {
            ConfigSource::System => &self.system_ast.syntax(),
            ConfigSource::Home => &self.home_ast.syntax(),
        };
        let fallback_module_value: bool = modules
            .get(self.selected_index)
            .map(|m| m.value.clone())
            .unwrap_or(false);

        let (new_node, new_ast, new_modules, new_module_value) = match modules
            .get(self.selected_index)
        {
            Some(module) => {
                let new_node: SyntaxNode = toggle_bool_at_path(node, module.path.as_str());

                let new_ast: Parse<Root> = Root::parse(&new_node.to_string().as_str());

                let new_modules: Vec<ConfigOption> =
                    collect_nix_options(&new_node, "", "".to_string(), self.current_file.clone());

                let new_module_value: bool = get_nix_value_by_path(&new_node, module.path.as_str())
                    .unwrap_or_else(|| module.value.clone());

                (new_node, new_ast, new_modules, new_module_value)
            }
            None => (
                node.clone(),
                Root::parse(&node.to_string().as_str()),
                Vec::new(),
                fallback_module_value,
            ),
        };

        (
            modules.to_vec(),
            new_node,
            new_ast,
            new_modules,
            new_module_value,
        )
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let vertical_layout: Layout = Layout::vertical([
            Constraint::Percentage(4),
            Constraint::Percentage(91),
            Constraint::Percentage(5),
        ]);

        let [title_area, content_area, footer_area] = vertical_layout.areas(area);

        {
            let title: Line<'_> =
                Line::from("GarandOS TUI".bold().italic().yellow()).left_aligned();

            title.render(title_area, buf);
        }

        {
            let modules: &Vec<ConfigOption> = match self.current_file {
                ConfigSource::System => &self.system_modules,
                ConfigSource::Home => &self.home_modules,
            };

            let mut lines: Vec<Line> = Vec::new();
            let mut last_category: String = String::new();

            for (idx, module) in modules.iter().enumerate() {
                if module.category != last_category {
                    lines.push("".into());
                    let cat_line = Line::from(vec![
                        Span::raw(format!("/* {} */", module.category))
                            .dark_gray()
                            .italic(),
                    ]);
                    lines.push(cat_line);
                    last_category = module.category.clone();
                }

                let checkbox: &'static str = if module.value { "☑" } else { "☐" };

                let mut line: Line<'_> =
                    Line::from(vec![Span::raw(format!("{} {}", checkbox, module.path))]);

                if idx == self.selected_index {
                    line = line.style(Color::Yellow)
                }

                lines.push(line);
            }

            let block_title: &'static str = match self.current_file {
                ConfigSource::System => "[ System Configuration ]",
                ConfigSource::Home => "[ Home Configuration ]",
            };

            let content: Paragraph<'_> = Paragraph::new(lines)
                .left_aligned()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(block_title.blue().bold()),
                )
                .scroll((self.selected_index as u16, 0));
            content.render(content_area, buf);
        }

        {
            let footer_line1: Line<'_> = Line::from(vec![
                Span::from("Use "),
                Span::from("↑/↓".italic().green()),
                Span::from(" to navigate, "),
                Span::from("Enter/Space".bold().yellow()),
                Span::from(" to toggle, "),
                Span::from("1/2 or ←/→".bold().blue()),
                Span::from(" to switch file, "),
                Span::from("q".bold().red()),
                Span::from(" to quit."),
            ]);
            let last_action: Span<'_> = if self.last_action_status.is_empty() {
                Span::from("Nothing changed")
            } else {
                self.last_action_status.clone().into()
            };
            let footer_line2: Line<'_> = Line::from(vec![
                Span::from("Last action: "),
                last_action.italic().green(),
                Span::from("."),
            ]);
            let footer: Paragraph<'_> =
                Paragraph::new(vec![footer_line1, footer_line2]).left_aligned();

            footer.render(footer_area, buf);
        }
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
