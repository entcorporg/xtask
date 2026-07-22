use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::util::naming::to_snake_case;

mod templates;

/// Dossiers vides du squelette (juste un `.gitkeep`).
const EMPTY_DIRS: &[&str] = &[
    "migrations",
    "src/infrastructure/services",
    "tests/architecture",
    "tests/feature",
    "tests/unit",
];

pub fn generate(raw_name: &str, dest_override: Option<PathBuf>) -> Result<()> {
    let snake = to_snake_case(raw_name);
    if snake.is_empty() {
        bail!("le nom du projet ne peut pas être vide");
    }

    let dest_parent = match dest_override {
        Some(p) => p,
        None => std::env::current_dir().context("impossible de lire le répertoire courant")?,
    };

    let project_dir = dest_parent.join(&snake);
    if project_dir.exists() {
        bail!("`{}` existe déjà, abandon", project_dir.display());
    }

    let api_dir = project_dir.join("api");

    for (relative_path, content) in templates::files() {
        let path = api_dir.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("échec de la création de {}", parent.display()))?;
        }
        fs::write(&path, content)
            .with_context(|| format!("échec de l'écriture de {}", path.display()))?;
    }

    for dir in EMPTY_DIRS {
        let path = api_dir.join(dir);
        fs::create_dir_all(&path)
            .with_context(|| format!("échec de la création de {}", path.display()))?;
        fs::write(path.join(".gitkeep"), "")
            .with_context(|| format!("échec de l'écriture de {}/.gitkeep", path.display()))?;
    }

    println!("projet `{snake}` créé dans {}", project_dir.display());
    println!(
        "-> cd {} && cargo xtask new domain <nom>",
        project_dir.display()
    );

    Ok(())
}