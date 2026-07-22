use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "xtask",
    about = "Outil de scaffolding pour le crate `api`",
    version
)]
pub struct Cli {
    /// Pour `new ...` : racine du repo cible (sinon détection automatique
    /// en remontant depuis le répertoire courant, à la recherche de
    /// `api/Cargo.toml`). Pour `init <name>` : dossier parent dans lequel
    /// créer le nouveau projet (par défaut le répertoire courant).
    /// Peut aussi être fourni via la variable d'env `XTASK_ROOT`.
    #[arg(long, short = 'r', global = true, env = "XTASK_ROOT")]
    pub root: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Génère un nouveau fichier source dans un projet existant (extractor, domain, ...)
    New {
        #[command(subcommand)]
        resource: NewCommands,
    },

    /// Crée un nouveau projet complet, sur le modèle de `laravel new`
    /// (arborescence api/ entière : bootstrap, config, http, infrastructure, routes, tests...)
    Init {
        /// Nom du nouveau projet
        name: String,
    },

    /// Met à jour xtask vers la dernière version publiée sur GitHub Releases
    /// (pipeline cargo-dist)
    #[command(name = "self-update", alias = "selfupdate")]
    SelfUpdate {
        /// N'affiche pas la confirmation avant d'installer
        #[arg(long, short = 'y')]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum NewCommands {
    /// Génère le squelette d'un extractor dans api/src/http/extractors
    #[command(alias = "extractors")]
    Extractor {
        /// Nom de l'extractor, ex: "auth" -> AuthExtractor
        name: String,
    },

    /// Génère un nouveau domaine (dtos/entities/errors/events/listeners/
    /// repositories/services) dans api/src/domains
    #[command(aliases = ["domains", "workspace", "workspaces"])]
    Domain {
        /// Nom du domaine, ex: "billing"
        name: String,
    },
}