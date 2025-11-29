use crossterm::{
    event,
    event::{KeyCode, KeyEvent, KeyEventKind},
};
use ratatui::{
    DefaultTerminal, Frame, Terminal,
    prelude::{Buffer, Constraint, CrosstermBackend, Layout, Rect, Stylize},
    style::{Color, Style},
    symbols::border,
    text::Line,
    widgets::{Block, Gauge, Widget},
};
use std::{
    io,
    sync::mpsc::{self, Sender},
    thread,
    time::Duration,
};

fn main() -> io::Result<()> {
    let mut terminal: Terminal<CrosstermBackend<io::Stdout>> = ratatui::init();
    let mut app: App = App {
        exit: false,
        progress_bar_color: Color::Green,
        background_progress: 0_f64,
    };

    let (event_tx, event_rx) = mpsc::channel::<Event>();

    let tx_to_input_events: Sender<Event> = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    let tx_to_background_events: Sender<Event> = event_tx.clone();
    thread::spawn(move || {
        run_background_thread(tx_to_background_events);
    });

    let app_result: Result<(), io::Error> = app.run(&mut terminal, event_rx);

    ratatui::restore();
    app_result
}

enum Event {
    Input(KeyEvent),
    Progress(f64),
}

fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        match event::read().unwrap() {
            event::Event::Key(key_event) => tx.send(Event::Input(key_event)).unwrap(),
            _ => {}
        }
    }
}

fn run_background_thread(tx: mpsc::Sender<Event>) {
    let mut progress: f64 = 0_f64;
    const INCREMENT: f64 = 0.01_f64;
    loop {
        thread::sleep(Duration::from_millis(100));
        progress += INCREMENT;
        progress = progress.min(1_f64);
        tx.send(Event::Progress(progress)).unwrap();
    }
}

pub struct App {
    exit: bool,
    progress_bar_color: Color,
    background_progress: f64,
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<Event>) -> io::Result<()> {
        while !self.exit {
            match rx.recv().unwrap() {
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
