pub fn mod_rs() -> String {
    r#"pub mod conf;
pub mod cors;
pub mod grpc_port;
pub mod http_address;
pub mod http_port;
pub mod state;
"#
    .to_string()
}

pub fn conf() -> String {
    r#"use std::net::IpAddr;

use crate::config::{
    cors::cors_allowed_origins_from_env, http_address::http_address_from_env,
    http_port::http_port_from_env,
};

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub address: IpAddr,
    pub app_env: String,   // "development" ou "production"
    pub log_level: String, // "info", "debug", "warn", etc.
    pub metrics_token: Option<String>,
    pub metrics_allow_open: bool,
    pub cors_allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        // Charge les variables du fichier .env s'il existe
        let _ = dotenvy::dotenv();
        // Lecture de APP_ENV (par défaut: "development")
        let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        // Lecture de RUST_LOG (par défaut: "info")
        let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

        Ok(Self {
            port: http_port_from_env(4002)?,
            address: http_address_from_env(IpAddr::from([0, 0, 0, 0]))?,
            metrics_token: std::env::var("METRICS_TOKEN")
                .ok()
                .filter(|s| !s.is_empty()),
            metrics_allow_open: std::env::var("METRICS_ALLOW_OPEN")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            cors_allowed_origins: cors_allowed_origins_from_env(),
            app_env,
            log_level,
        })
    }
}
"#
    .to_string()
}

pub fn cors() -> String {
    r#"pub fn cors_allowed_origins_from_env() -> Vec<String> {
    std::env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}
"#
    .to_string()
}

pub fn grpc_port() -> String {
    r#"pub fn grpc_port_from_env(default: u16) -> anyhow::Result<u16> {
    match std::env::var("GRPC_PORT") {
        Ok(v) => v
            .parse()
            .map_err(|e| anyhow::anyhow!("GRPC_PORT invalide: {e}")),
        Err(_) => Ok(default),
    }
}
"#
    .to_string()
}

pub fn http_address() -> String {
    r#"pub fn http_address_from_env<T>(default: T) -> anyhow::Result<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    match std::env::var("BIND_ADDRESS") {
        Ok(v) => v
            .parse::<T>()
            .map_err(|e| anyhow::anyhow!("BIND_ADDRESS invalide: {e}")),
        Err(_) => Ok(default),
    }
}
"#
    .to_string()
}

pub fn http_port() -> String {
    r#"pub fn http_port_from_env(default: u16) -> anyhow::Result<u16> {
    match std::env::var("PORT") {
        Ok(v) => v.parse().map_err(|e| anyhow::anyhow!("PORT invalide: {e}")),
        Err(_) => Ok(default),
    }
}
"#
    .to_string()
}

pub fn state() -> String {
    r#"use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::config::conf::Config;

// Optionnel: Décommente si tu utilises SQLx pour ta DB
// use sqlx::PgPool;

/// État partagé de l'application injecté dans les handlers HTTP.
///
/// La structure derive `Clone` pour être distribuée facilement
/// aux différents threads du serveur Web (Axum/Actix).
#[derive(Clone, Debug)]
pub struct AppState {
    /// Configuration immuable de l'application enveloppée dans un Arc
    pub config: Arc<Config>,
    pub metrics_token_hash: Option<[u8; 32]>,
    // Pool de connexion à la base de données (ex: PgPool est déjà un Arc sous le capot)
    // pub db: PgPool,

    // Ajoute ici tes autres clients d'infrastructure si nécessaire:
    // pub stripe_client: Arc<StripeClient>,
}

impl AppState {
    /// Instancie un nouvel AppState à partir de la configuration globale.
    pub fn new(config: Config /*, db: PgPool */) -> anyhow::Result<Self> {
        let metrics_token_hash = config
            .metrics_token
            .as_deref()
            .map(|t| Sha256::digest(t.as_bytes()).into());

        Ok(Self {
            config: Arc::new(config),
            metrics_token_hash,
            // db,
        })
    }
}
"#
    .to_string()
}
