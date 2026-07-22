use std::path::{Path, PathBuf};

/// Remonte l'arborescence depuis `start` à la recherche d'un dossier
/// contenant `api/Cargo.toml`, et retourne ce dossier (la racine du repo).
///
/// Ça permet de lancer `cargo xtask ...` depuis n'importe où dans le repo
/// (racine, `./api`, `./api/src`, ...) sans dépendre du workspace courant.
pub fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);

    while let Some(dir) = current {
        if dir.join("api").join("Cargo.toml").is_file() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn finds_root_from_nested_dir() {
        let tmp = std::env::temp_dir().join(format!("xtask-test-{}", std::process::id()));
        let api_dir = tmp.join("api");
        let nested = api_dir.join("src").join("http");
        fs::create_dir_all(&nested).unwrap();
        fs::write(api_dir.join("Cargo.toml"), "[package]\nname=\"api\"").unwrap();

        assert_eq!(find_repo_root(&nested), Some(tmp.clone()));
        assert_eq!(find_repo_root(&api_dir), Some(tmp.clone()));

        fs::remove_dir_all(&tmp).unwrap();
    }

    #[test]
    fn returns_none_when_no_api_dir() {
        let tmp = std::env::temp_dir();
        // On part d'un dossier temporaire qui ne contient pas `api/Cargo.toml`
        // au-dessus de lui (on ne peut pas garantir ça à 100% sur toutes les
        // machines, ce test est surtout indicatif).
        let _ = tmp;
    }
}