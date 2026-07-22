pub fn content() -> String {
    r#"use api::{bootstrap, config::conf::Config};
use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Chargement des variables d'environnement
    let config = Config::from_env()?;

    // 2. Initialisation du serveur & des routes via le module bootstrap
    bootstrap::run(config).await?;

    Ok(())
}
"#
    .to_string()
}
