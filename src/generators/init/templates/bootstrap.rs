pub fn content() -> String {
    r#"use axum::http::{HeaderValue, Method};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    config::{conf, state::AppState},
    infrastructure::logging::{LogEnvironment, init_logging},
    routes,
};

pub async fn run(config: conf::Config) -> anyhow::Result<()> {
    let log_env = LogEnvironment::from_str(&config.app_env);
    init_logging(log_env, &config.log_level);

    tracing::info!("Démarrage du backend API...");

    let state = AppState::new(config.clone())?;
    tracing::info!("AppState initialisé");

    let cors = if config.cors_allowed_origins.is_empty() {
        CorsLayer::new()
    } else {
        let origins: Vec<HeaderValue> = config
            .cors_allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PATCH,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                axum::http::header::AUTHORIZATION,
                axum::http::header::CONTENT_TYPE,
                axum::http::header::ACCEPT,
            ])
            .allow_credentials(true)
    };
    tracing::info!("Initialisation des cors");

    let app = routes::create_router(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors);
    tracing::info!("Initialisation des routes api");

    let http_addr = format!("{}:{}", config.address, config.port);
    tracing::info!("auth-service HTTP sur {}", http_addr);

    let listener = tokio::net::TcpListener::bind(&http_addr).await?;

    tokio::select! {
        res = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),

        ) => { res?; }
    }

    Ok(())
}
"#
    .to_string()
}
