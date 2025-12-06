use garandos_tui::{
    app::{App, Event, handle_input_events},
    cli::get_modules,
    nix::{ConfigSource, NixModules},
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use std::{
    io::{Result, Stdout},
    sync::mpsc::{self, Sender},
    thread,
};

fn main() -> Result<()> {
    let nix_modules: NixModules = get_modules();
    let mut terminal: Terminal<CrosstermBackend<Stdout>> = ratatui::init();
    let mut app: App = App {
        exit: false,
        system_path: nix_modules.system_path,
        home_path: nix_modules.home_path,
        system_ast: nix_modules.system_ast,
        home_ast: nix_modules.home_ast,
        system_modules: nix_modules.system_modules,
        home_modules: nix_modules.home_modules,
        current_file: ConfigSource::System,
        selected_index: 0_usize,
        last_action_status: "".to_string(),
    };

    let (event_tx, event_rx) = mpsc::channel::<Event>();

    let tx_to_input_events: Sender<Event> = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    let app_result: Result<()> = app.run(&mut terminal, event_rx);
    ratatui::restore();
    app_result
}
