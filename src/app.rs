use crossterm::{
    event,
    event::{KeyCode, KeyEvent, KeyEventKind},
};
use ratatui::{
    DefaultTerminal, Frame,
    prelude::{Buffer, Constraint, Layout, Rect, Stylize},
    style::{Color, Style},
    symbols::border,
    text::Line,
    widgets::{Block, Gauge, Widget},
};
use std::{io, sync::mpsc, thread, time::Duration};

pub enum Event {
    Input(KeyEvent),
    Progress(f64),
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

pub fn run_background_thread(tx: mpsc::Sender<Event>) {
    let mut progress: f64 = 0_f64;
    const INCREMENT: f64 = 0.01_f64;
    loop {
        thread::sleep(Duration::from_millis(100));
        progress += INCREMENT;
        progress = progress.min(1_f64);
        if tx.send(Event::Progress(progress)).is_err() {
            break;
        }
    }
}

pub struct App {
    pub exit: bool,
    pub progress_bar_color: Color,
    pub background_progress: f64,
}

impl App {
    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<Event>,
    ) -> io::Result<()> {
        while !self.exit {
            let event: Event = match rx.recv() {
                Ok(ev) => ev,
                Err(_) => break,
            };
            match event {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
                Event::Progress(progress) => self.background_progress = progress,
            }
            terminal.draw(|frame: &mut Frame<'_>| self.draw(frame))?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame<'_>) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char('q') {
            self.exit = true;
        } else if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char('c') {
            if self.progress_bar_color == Color::Green {
                self.progress_bar_color = Color::Red;
            } else {
                self.progress_bar_color = Color::Green;
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
        let vertical_layout: Layout =
            Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(80)]);
        let [title_area, gauge_area] = vertical_layout.areas(area);

        Line::from("Process overview")
            .bold()
            .render(title_area, buf);

        let instructions: Line<'_> = Line::from(vec![
            " Change color ".into(),
            "<C>".blue().bold(),
            " Quit ".into(),
            "<Q>".blue().bold(),
        ])
        .centered();

        let block: Block<'_> = Block::bordered()
            .title(Line::from(" Background processes "))
            .title_bottom(instructions)
            .border_set(border::THICK);

        let progress_bar: Gauge<'_> = Gauge::default()
            .gauge_style(Style::default().fg(self.progress_bar_color))
            .block(block)
            .label(format!(
                "Process Bar: {:.2}%",
                self.background_progress * 100_f64
            ))
            .ratio(self.background_progress);

        progress_bar.render(
            Rect {
                x: gauge_area.left(),
                y: gauge_area.top(),
                width: gauge_area.width,
                height: 3,
            },
            buf,
        );
    }
}
