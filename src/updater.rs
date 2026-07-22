use std::{
    fs,
    path::{Path, PathBuf},
};

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

fn replace_binary(new_exe: &Path, install_path: &Path) -> Result<()> {
    if let Some(parent) = install_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("impossible de créer le dossier {}", parent.display())
        })?;
    }

    fs::copy(new_exe, install_path).with_context(|| {
        format!(
            "impossible de copier le binaire mis à jour de {} vers {}",
            new_exe.display(),
            install_path.display()
        )
    })?;

    #[cfg(unix)]
    {
        let source_perms = fs::metadata(new_exe)?.permissions();
        fs::set_permissions(install_path, source_perms).with_context(|| {
            format!(
                "impossible d'appliquer les permissions au binaire {}",
                install_path.display()
            )
        })?;
    }

    Ok(())
}

fn temp_install_path(current_exe: &Path) -> Result<PathBuf> {
    let parent = current_exe.parent().context("le binaire courant n'a pas de dossier parent")?;
    let file_name = current_exe
        .file_name()
        .context("le binaire courant n'a pas de nom de fichier")?
        .to_string_lossy();
    Ok(parent.join(format!("{file_name}.new")))
}

pub fn run(yes: bool) -> Result<()> {
    let (owner, name) = repo_owner_and_name()?;
    let current_version = cargo_crate_version!();

    println!("xtask v{current_version} — recherche d'une mise à jour sur {owner}/{name}...");

    let target = self_update::get_target();
    
    // Gère Windows (.exe) vs Linux/macOS
    let bin_name_ext = if cfg!(windows) {
        format!("{BIN_NAME}.exe")
    } else {
        BIN_NAME.to_string()
    };

    // Teste sans le préfixe de sous-dossier OU avec `./` selon le comportement de tar
    let bin_path = format!("{BIN_NAME}-{target}/{bin_name_ext}");

    let current_exe = std::env::current_exe().context("impossible de localiser le binaire courant")?;
    let temp_install_path = temp_install_path(&current_exe)?;

    let status = self_update::backends::github::Update::configure()
        .repo_owner(owner)
        .repo_name(name)
        .bin_name(&bin_name_ext) // <--- Utilise l'extension .exe sur Windows
        .bin_path_in_archive(&bin_path)
        .bin_install_path(&temp_install_path)
        .show_download_progress(true)
        .no_confirm(yes)
        .current_version(current_version)
        .build()
        .context("échec de la configuration de self_update")?
        .update()
        .context("échec de la mise à jour")?;

    replace_binary(&temp_install_path, &current_exe).with_context(|| {
        format!(
            "impossible de remplacer le binaire courant {} avec {}",
            current_exe.display(),
            temp_install_path.display()
        )
    })?;

    let _ = fs::remove_file(&temp_install_path);

    match status {
        self_update::Status::UpToDate(v) => println!("déjà à jour (v{v})"),
        self_update::Status::Updated(v) => {
            println!("mis à jour vers v{v} — relance `cargo xtask --version` pour vérifier");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{replace_binary, temp_install_path};
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::{
        fs,
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn replace_binary_copies_contents_and_permissions() {
        let temp_dir = std::env::temp_dir().join(format!(
            "xtask-updater-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        let source_path = temp_dir.join("source.bin");
        let target_path = temp_dir.join("target.bin");

        let source_bytes = b"updated-binary";
        fs::write(&source_path, source_bytes).unwrap();
        fs::write(&target_path, b"old-binary").unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&source_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&source_path, perms).unwrap();
        }

        replace_binary(&source_path, &target_path).unwrap();

        let updated = fs::read(&target_path).unwrap();
        assert_eq!(updated, source_bytes);

        #[cfg(unix)]
        {
            let target_meta = fs::metadata(&target_path).unwrap();
            let mode = target_meta.permissions().mode() & 0o777;
            assert_eq!(mode, 0o755);
        }
    }

    #[test]
    fn temp_install_path_uses_a_sibling_file() {
        let current_exe = Path::new("/tmp/cargo-xtask");
        let temp_path = temp_install_path(current_exe).unwrap();
        assert_eq!(temp_path, Path::new("/tmp/cargo-xtask.new"));
    }
}
