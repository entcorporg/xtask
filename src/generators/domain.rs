use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::util::naming::to_snake_case;
use crate::util::repo_root::find_repo_root;

/// Sous-modules standards d'un domaine, sur le modèle de `domains/auth`
/// et `domains/user`.
const SUBMODULES: &[&str] = &[
    "dtos",
    "entities",
    "errors",
    "events",
    "listeners",
    "repositories",
    "services",
];

pub fn generate(raw_name: &str, root_override: Option<PathBuf>) -> Result<()> {
    let snake = to_snake_case(raw_name);
    if snake.is_empty() {
        bail!("le nom du workspace ne peut pas être vide");
    }

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

    let domains_dir = repo_root.join("api/src/domains");
    if !domains_dir.is_dir() {
        bail!("le dossier `{}` est introuvable", domains_dir.display());
    }

    let workspace_dir = domains_dir.join(&snake);
    if workspace_dir.exists() {
        bail!("`{}` existe déjà, abandon", workspace_dir.display());
    }

    fs::create_dir_all(&workspace_dir)
        .with_context(|| format!("échec de la création de {}", workspace_dir.display()))?;

    for sub in SUBMODULES {
        let sub_dir = workspace_dir.join(sub);
        fs::create_dir_all(&sub_dir)
            .with_context(|| format!("échec de la création de {}", sub_dir.display()))?;

        let mod_path = sub_dir.join("mod.rs");
        fs::write(&mod_path, render_submodule(sub, &snake))
            .with_context(|| format!("échec de l'écriture de {}", mod_path.display()))?;
        println!("créé {}", mod_path.display());
    }

    let domain_mod_path = workspace_dir.join("mod.rs");
    fs::write(&domain_mod_path, render_domain_mod())
        .with_context(|| format!("échec de l'écriture de {}", domain_mod_path.display()))?;
    println!("créé {}", domain_mod_path.display());

    update_domains_mod(&domains_dir, &snake)?;

    println!("domaine `{snake}` créé dans {}", workspace_dir.display());

    Ok(())
}

fn render_submodule(sub: &str, domain: &str) -> String {
    format!("//! TODO: {sub} du domaine `{domain}`\n")
}

fn render_domain_mod() -> String {
    let mut out = String::new();
    for sub in SUBMODULES {
        out.push_str(&format!("pub mod {sub};\n"));
    }
    out
}

/// Ajoute `pub mod {snake};` dans api/src/domains/mod.rs, regroupé avec
/// les autres déclarations `pub mod`.
fn update_domains_mod(domains_dir: &Path, snake: &str) -> Result<()> {
    let mod_path = domains_dir.join("mod.rs");
    let existing = fs::read_to_string(&mod_path).unwrap_or_default();

    let mod_line = format!("pub mod {snake};");
    if existing.contains(&mod_line) {
        println!("`{mod_line}` déjà présent dans mod.rs, on passe");
        return Ok(());
    }

    let mut lines: Vec<String> = existing.lines().map(str::to_string).collect();

    let insert_at = lines
        .iter()
        .rposition(|l| l.trim_start().starts_with("pub mod "))
        .map(|i| i + 1)
        .unwrap_or(lines.len());
    lines.insert(insert_at, mod_line);

    let mut new_content = lines.join("\n");
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }

    fs::write(&mod_path, new_content)
        .with_context(|| format!("échec de la mise à jour de {}", mod_path.display()))?;
    println!("mis à jour {}", mod_path.display());

    Ok(())
}