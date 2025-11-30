use rnix::{Parse, Root};
use std::fs;

mod nix;

fn main() {
    let content: String =
        fs::read_to_string("src/test.nix").expect("Nie można odczytać pliku src/test.nix");

    let ast: Parse<Root> = Root::parse(&content);

    if !ast.errors().is_empty() {
        eprintln!("Błędy parsowania:");
        for error in ast.errors() {
            eprintln!("  - {}", error);
        }
        eprintln!();
    }

    let options: Vec<(String, bool)> = nix::collect_nix_options_with_path(&ast.syntax(), "");
    for (path, value) in options {
        println!("{} = {};", path, value);
    }
}
