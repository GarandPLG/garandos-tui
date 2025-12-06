use crate::nix::{NixModules, build_nix_modules};
use clap::Parser;
use std::path::PathBuf;

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

pub fn get_modules() -> NixModules {
    let args: Cli = Cli::parse();
    build_nix_modules(args.sf, args.hf)
}
