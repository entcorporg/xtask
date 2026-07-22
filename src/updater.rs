use std::{
    fs,
    path::Path,
    process::Command,
};

use tar::Archive;
use xz2::read::XzDecoder;

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

fn install_and_replace(new_exe: &Path, install_path: &Path) -> Result<()> {
    let temp_install_path = install_path.with_extension("new");
    replace_binary(new_exe, &temp_install_path)?;

    #[cfg(unix)]
    {
        let helper = std::env::current_exe().context("impossible de localiser le binaire courant")?;
        let _ = Command::new(&helper)
            .arg("self-update-helper")
            .arg(&temp_install_path)
            .arg(install_path)
            .spawn()
            .context("impossible de démarrer le helper de mise à jour")?;
    }

    Ok(())
}

fn extract_archive_file(archive_path: &Path, into_dir: &Path, file_to_extract: &str) -> Result<()> {
    let archive_file = fs::File::open(archive_path).with_context(|| {
        format!("impossible d'ouvrir l'archive {}", archive_path.display())
    })?;
    let mut archive = Archive::new(XzDecoder::new(archive_file));
    let mut matching_entry = None;

    for entry in archive.entries().context("impossible de lire l'archive tar.xz")? {
        let entry = entry.context("impossible de lire une entrée de l'archive")?;
        let entry_path = entry
            .path()
            .context("impossible de lire le chemin d'une entrée")?
            .to_path_buf();
        if entry_path == Path::new(file_to_extract) {
            matching_entry = Some(entry);
            break;
        }
    }

    let mut matching_entry = matching_entry.context(format!(
        "impossible de trouver {} dans l'archive {}",
        file_to_extract,
        archive_path.display()
    ))?;

    matching_entry.unpack_in(into_dir).with_context(|| {
        format!(
            "impossible d'extraire {} depuis {} vers {}",
            file_to_extract,
            archive_path.display(),
            into_dir.display()
        )
    })?;

    Ok(())
}

pub fn run(yes: bool) -> Result<()> {
    let (owner, name) = repo_owner_and_name()?;
    let current_version = cargo_crate_version!();

    println!("xtask v{current_version} — recherche d'une mise à jour sur {owner}/{name}...");

    let target = self_update::get_target();

    let bin_name_ext = if cfg!(windows) {
        format!("{BIN_NAME}.exe")
    } else {
        BIN_NAME.to_string()
    };

    let bin_path = format!("{BIN_NAME}-{target}/{bin_name_ext}");
    let current_exe = std::env::current_exe().context("impossible de localiser le binaire courant")?;

    let updater = self_update::backends::github::Update::configure()
        .repo_owner(owner)
        .repo_name(name)
        .bin_name(&bin_name_ext)
        .show_download_progress(true)
        .no_confirm(yes)
        .current_version(current_version)
        .build()
        .context("échec de la configuration de self_update")?;

    let releases = updater
        .get_latest_releases(&current_version)
        .context("échec de récupération des releases")?;

    let release = releases.first().cloned().context("aucune release compatible n'a été trouvée")?;
    let target_asset = release
        .asset_for(&target, None)
        .context("aucun asset de release compatible n'a été trouvé")?;
    let download_url = format!(
        "https://github.com/{owner}/{name}/releases/download/v{}/{}",
        release.version,
        target_asset.name
    );

    let temp_dir = std::env::temp_dir().join(format!(
        "xtask-update-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&temp_dir)?;

    let archive_path = temp_dir.join(&target_asset.name);
    let mut archive_file = fs::File::create(&archive_path)?;

    println!("Downloading...");
    let mut download = self_update::Download::from_url(&download_url);
    let headers = updater.api_headers(&updater.auth_token())?;
    download.set_headers(headers);
    download.download_to(&mut archive_file)?;
    drop(archive_file);

    println!("Extracting archive...");
    extract_archive_file(&archive_path, &temp_dir, &bin_path)?;

    let new_exe = temp_dir.join(&bin_path);
    install_and_replace(&new_exe, &current_exe).with_context(|| {
        format!(
            "impossible de remplacer le binaire courant {} avec {}",
            current_exe.display(),
            new_exe.display()
        )
    })?;

    println!(
        "mis à jour vers v{} — relance `cargo xtask --version` pour vérifier",
        release.version
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::replace_binary;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::{
        fs,
        io::Write,
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
    fn extract_archive_file_reads_tar_xz_entries() {
        let temp_dir = std::env::temp_dir().join(format!(
            "xtask-updater-archive-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        let archive_path = temp_dir.join("archive.tar.xz");
        let archive_file = fs::File::create(&archive_path).unwrap();
        let mut archived = std::io::BufWriter::new(archive_file);
        archived.write_all(b"not a real tar.xz archive").unwrap();
        drop(archived);

        let result = super::extract_archive_file(&archive_path, &temp_dir, "nested/bin");
        assert!(result.is_err());
    }
}
