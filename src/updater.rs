use anyhow::{Context, Result, bail};
use self_update::cargo_crate_version;

/// Doit correspondre au nom du binaire publié par `dist` (voir Cargo.toml
/// -> [[bin]] name = "cargo-xtask" et le nom des archives release).
const BIN_NAME: &str = "cargo-xtask";

/// Lit `owner/repo` depuis le champ `repository` de xtask/Cargo.toml
/// (ex: "https://github.com/acme/backend" -> ("acme", "backend")).
fn repo_owner_and_name() -> Result<(&'static str, &'static str)> {
    let repo = env!("CARGO_PKG_REPOSITORY");
    if repo.is_empty() {
        bail!(
            "le champ `repository` de xtask/Cargo.toml est vide — renseigne-le avec \
             l'URL GitHub du repo (ex: \"https://github.com/acme/backend\") pour \
             activer `cargo xtask self-update`"
        );
    }

    let trimmed = repo.trim_end_matches('/').trim_end_matches(".git");
    let mut parts = trimmed.rsplit('/');
    let name = parts
        .next()
        .context("impossible de parser le nom du repo depuis `repository`")?;
    let owner = parts
        .next()
        .context("impossible de parser le owner du repo depuis `repository`")?;

    Ok((owner, name))
}

pub fn run(yes: bool) -> Result<()> {
    let (owner, name) = repo_owner_and_name()?;
    let current_version = cargo_crate_version!();

    println!("xtask v{current_version} — recherche d'une mise à jour sur {owner}/{name}...");

    let target = self_update::get_target();
    let bin_path = format!("{BIN_NAME}-{target}/{BIN_NAME}");

    let status = self_update::backends::github::Update::configure()
        .repo_owner(owner)
        .repo_name(name)
        .bin_name(BIN_NAME)
        .bin_path_in_archive(&bin_path) // <-- Indique à self_update d'aller chercher le binaire dans le sous-dossier
        .show_download_progress(true)
        .no_confirm(yes)
        .current_version(current_version)
        .build()
        .context("échec de la configuration de self_update")?
        .update()
        .context("échec de la mise à jour")?;

    match status {
        self_update::Status::UpToDate(v) => println!("déjà à jour (v{v})"),
        self_update::Status::Updated(v) => {
            println!("mis à jour vers v{v} — relance `cargo xtask --version` pour vérifier");
        }
    }

    Ok(())
}
