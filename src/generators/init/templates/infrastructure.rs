pub fn mod_rs() -> String {
    "pub mod logging;\n".to_string()
}

pub fn logging_mod() -> String {
    r#"use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

/// Configuration de l'environnement de logging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogEnvironment {
    Development,
    Production,
}

impl LogEnvironment {
    pub fn from_str(env: &str) -> Self {
        match env.to_lowercase().as_str() {
            "production" | "prod" => Self::Production,
            _ => Self::Development,
        }
    }
}

/// Initialise le système de tracing/logging global de l'application
pub fn init_logging(env: LogEnvironment, default_level: &str) {
    // 1. Définition des filtres via RUST_LOG ou valeur par défaut
    // Exemple de filtre: "info,api=debug,tower_http=debug"
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    // 2. Formatage selon l'environnement
    match env {
        LogEnvironment::Production => {
            // Prod: Format JSON structuré (compatible Datadog, ELK, Loki, etc.)
            let json_layer = tracing_subscriber::fmt::layer()
                .json()
                .flatten_event(true)
                .with_current_span(true)
                .with_target(true);

            Registry::default().with(env_filter).with(json_layer).init();
        }
        LogEnvironment::Development => {
            // Dev: Pretty-print lisible en terminal avec couleurs
            let fmt_layer = tracing_subscriber::fmt::layer()
                .pretty()
                .with_thread_names(true)
                .with_line_number(true)
                .with_target(true);

            Registry::default().with(env_filter).with(fmt_layer).init();
        }
    }

    tracing::info!(
        environment = ?env,
        "Système de logging initialisé avec succès."
    );
}
"#
    .to_string()
}
