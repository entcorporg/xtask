mod cli;
mod generators;
mod updater;
mod util;

use std::process::exit;

use clap::Parser;

use cli::{Cli, Commands, NewCommands};

fn main() {
    // Quand cargo appelle `cargo xtask ...`, il exécute le binaire
    // `cargo-xtask` en lui passant "xtask" comme premier argument
    // (ex: `cargo-xtask xtask new extractor auth`). On le retire avant
    // de parser, pour que ça marche aussi bien via `cargo xtask ...`
    // que via un appel direct `cargo-xtask ...`.
    let mut args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("xtask") {
        args.remove(1);
    }

    let cli = Cli::parse_from(args);
    let root = cli.root.clone();

    let result = match cli.command {
        Commands::New { resource } => match resource {
            NewCommands::Extractor { name } => generators::extractor::generate(&name, root),
            NewCommands::Domain { name } => generators::domain::generate(&name, root),
        },
        Commands::Init { name } => generators::init::generate(&name, root),
        Commands::SelfUpdate { yes } => updater::run(yes),
    };

    if let Err(err) = result {
        eprintln!("erreur: {err:#}");
        exit(1);
    }
}