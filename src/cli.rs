use crate::nix::{ConfigOption, ConfigSource, collect_nix_options};
use clap::Parser;
use rnix::{Parse, Root};
use std::{fs, path::PathBuf};

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Potrzebne pliki znajdziesz w ~/garandos/hosts/<Twój-Host>/",
    long_about = "Potrzebne pliki znajdziesz w ~/garandos/hosts/<Twój-Host>/"
)]
pub struct Cli {
    #[arg(
        long,
        help = "Ścieżka do pliku system-modules.nix",
        value_name = "SYSTEM_MODULES"
    )]
    pub sf: PathBuf,
    #[arg(
        long,
        help = "Ścieżka do pliku home-modules.nix",
        value_name = "HOME_MODULES"
    )]
    pub hf: PathBuf,
}

pub struct NixModules {
    pub system_modules: Vec<ConfigOption>,
    pub home_modules: Vec<ConfigOption>,
}

pub fn get_modules() -> NixModules {
    let args: Cli = Cli::parse();

    let system_file: String = fs::read_to_string(&args.sf).expect("Failed to read system file");
    let home_file: String = fs::read_to_string(&args.hf).expect("Failed to read home file");

    let system_ast: Parse<Root> = get_ast(&system_file);
    let home_ast: Parse<Root> = get_ast(&home_file);

    let system_modules: Vec<ConfigOption> =
        collect_nix_options(&system_ast.syntax(), "", ConfigSource::System);
    let home_modules: Vec<ConfigOption> =
        collect_nix_options(&home_ast.syntax(), "", ConfigSource::Home);

    NixModules {
        system_modules,
        home_modules,
    }
}

fn get_ast(file: &str) -> Parse<Root> {
    let ast: Parse<Root> = Root::parse(&file);

    if !ast.errors().is_empty() {
        eprintln!("Błędy parsowania:");
        for error in ast.errors() {
            eprintln!("  - {}", error);
        }
        panic!("Błąd z parsowaniem pliku .nix")
    }

    ast
}
