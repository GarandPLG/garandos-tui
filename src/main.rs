use garandos_tui::{
    app::{App, Event, handle_input_events, run_background_thread},
    // nix::{collect_nix_options, get_nix_value_by_path, toggle_bool_at_path},
};
use ratatui::{Terminal, prelude::CrosstermBackend, style::Color};
// use rnix::{Parse, Root, SyntaxNode};
// use std::fs;
use std::{
    io,
    sync::mpsc::{self, Sender},
    thread,
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

// fn main() {
//     let content: String =
//         fs::read_to_string("src/test.nix").expect("Nie można odczytać pliku src/test.nix");

//     let ast: Parse<Root> = Root::parse(&content);

//     if !ast.errors().is_empty() {
//         eprintln!("Błędy parsowania:");
//         for error in ast.errors() {
//             eprintln!("  - {}", error);
//         }
//         eprintln!();
//     }

//     let node: SyntaxNode = ast.syntax();
//     let options: Vec<(String, String, bool)> = collect_nix_options(&node);

//     const OPTION: &str = "flatpak.enable";

//     println!("Options:");
//     for (_, path, value) in options {
//         if path == OPTION {
//             println!("{} = {};", path, value);
//         }
//     }

//     // if let Some(value) = get_nix_value_by_path(&node, OPTION) {
//     //     println!("Flatseal ma wartość: {}", value);
//     // }

//     let new_node: SyntaxNode = toggle_bool_at_path(&node, OPTION);
//     let new_options: Vec<(String, String, bool)> = collect_nix_options(&new_node);

//     println!("\nNew Options:");
//     for (_, path, value) in new_options {
//         if path == OPTION {
//             println!("{} = {};", path, value);
//         }
//     }

//     fs::write("src/test.nix", new_node.to_string()).expect("Nie można zapisać tego pliku");
// }
