use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::util::naming::{to_pascal_case, to_snake_case};
use crate::util::repo_root::find_repo_root;

pub fn generate(raw_name: &str, root_override: Option<PathBuf>) -> Result<()> {
    let snake = to_snake_case(raw_name);
    if snake.is_empty() {
        bail!("le nom de l'extractor ne peut pas être vide");
    }
    let struct_name = format!("{}Extractor", to_pascal_case(raw_name));

    let repo_root = match root_override {
        Some(root) => root,
        None => {
            let cwd =
                std::env::current_dir().context("impossible de lire le répertoire courant")?;
            find_repo_root(&cwd).with_context(|| {
                format!(
                    "impossible de localiser `api/Cargo.toml` en remontant depuis `{}` — \
                     précise la racine du repo avec `--root <chemin>` (ou la variable \
                     d'env `XTASK_ROOT`)",
                    cwd.display()
                )
            })?
        }
    };

    let dir: PathBuf = repo_root.join("api/src/http/extractors");
    if !dir.is_dir() {
        bail!("le dossier `{}` est introuvable", dir.display());
    }

    let file_path: PathBuf = dir.join(format!("{snake}.rs"));
    if file_path.exists() {
        bail!("`{}` existe déjà, abandon", file_path.display());
    }

    fs::write(&file_path, render_template(&struct_name))
        .with_context(|| format!("échec de l'écriture de {}", file_path.display()))?;
    println!("créé {}", file_path.display());

    update_mod_rs(&dir, &snake, &struct_name)?;

    Ok(())
}

fn render_template(struct_name: &str) -> String {
    format!(
        r#"use axum::{{extract::FromRequestParts, http::request::Parts}};

use crate::http::responses::ApiError; // TODO: adapte l'import si besoin

/// TODO: documenter ce que `{struct_name}` extrait et pourquoi.
#[derive(Debug, Clone)]
pub struct {struct_name} {{
    // TODO: ajouter les champs
}}

impl<S> FromRequestParts<S> for {struct_name}
where
    S: Send + Sync,
{{
    type Rejection = ApiError; // TODO: adapte le type de rejection

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {{
        // TODO: implémenter la logique d'extraction
        let _ = parts;
        todo!("implémenter {struct_name}::from_request_parts")
    }}
}}
"#
    )
}

/// Ajoute `mod {snake};` et `pub use {snake}::{struct_name};` dans mod.rs,
/// en les regroupant avec les lignes existantes du même type.
fn update_mod_rs(dir: &Path, snake: &str, struct_name: &str) -> Result<()> {
    let mod_path = dir.join("mod.rs");
    let existing = fs::read_to_string(&mod_path).unwrap_or_default();

    let mod_line = format!("mod {snake};");
    let use_line = format!("pub use {snake}::{struct_name};");

    if existing.contains(&mod_line) {
        println!("`{mod_line}` déjà présent dans mod.rs, on passe");
        return Ok(());
    }

    let mut lines: Vec<String> = existing.lines().map(str::to_string).collect();

    let mod_insert_at = lines
        .iter()
        .rposition(|l| l.trim_start().starts_with("mod "))
        .map(|i| i + 1)
        .unwrap_or(0);
    lines.insert(mod_insert_at, mod_line);

    let use_insert_at = lines
        .iter()
        .rposition(|l| l.trim_start().starts_with("pub use "))
        .map(|i| i + 1)
        .unwrap_or(lines.len());
    lines.insert(use_insert_at, use_line);

    let mut new_content = lines.join("\n");
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }

    fs::write(&mod_path, new_content)
        .with_context(|| format!("échec de la mise à jour de {}", mod_path.display()))?;
    println!("mis à jour {}", mod_path.display());

    Ok(())
}