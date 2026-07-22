mod bootstrap;
mod cargo;
mod config;
mod domains;
mod env;
mod http;
mod infrastructure;
mod lib_rs;
mod main_rs;
mod routes;

/// Liste ordonnée `(chemin relatif à api/, contenu)` de tous les fichiers
/// du squelette généré par `cargo xtask init`.
pub fn files() -> Vec<(&'static str, String)> {
    vec![
        ("Cargo.toml", cargo::content()),
        (".env.example", env::content()),
        ("src/main.rs", main_rs::content()),
        ("src/lib.rs", lib_rs::content()),
        ("src/bootstrap/mod.rs", bootstrap::content()),
        ("src/config/mod.rs", config::mod_rs()),
        ("src/config/conf.rs", config::conf()),
        ("src/config/cors.rs", config::cors()),
        ("src/config/grpc_port.rs", config::grpc_port()),
        ("src/config/http_address.rs", config::http_address()),
        ("src/config/http_port.rs", config::http_port()),
        ("src/config/state.rs", config::state()),
        ("src/domains/mod.rs", domains::mod_rs()),
        ("src/http/mod.rs", http::mod_rs()),
        ("src/http/extractors/mod.rs", http::extractors_mod()),
        ("src/http/handlers/mod.rs", http::handlers_mod()),
        ("src/http/handlers/api/mod.rs", http::handlers_api_mod()),
        ("src/http/handlers/api/health.rs", http::health()),
        ("src/http/handlers/api/livez.rs", http::livez()),
        ("src/http/handlers/api/readyz.rs", http::readyz()),
        ("src/http/handlers/api/v1/mod.rs", http::v1_mod()),
        (
            "src/http/handlers/api/v1/metrics/mod.rs",
            http::v1_metrics_mod(),
        ),
        (
            "src/http/handlers/api/v1/metrics/prometheus_endpoint.rs",
            http::v1_metrics_prometheus_endpoint(),
        ),
        ("src/http/middlewares/mod.rs", http::middlewares_mod()),
        ("src/http/middlewares/metrics.rs", http::middlewares_metrics()),
        ("src/http/responses/mod.rs", http::responses_mod()),
        ("src/infrastructure/mod.rs", infrastructure::mod_rs()),
        (
            "src/infrastructure/logging/mod.rs",
            infrastructure::logging_mod(),
        ),
        ("src/routes/mod.rs", routes::content()),
    ]
}