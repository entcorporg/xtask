pub fn mod_rs() -> String {
    r#"pub mod extractors;
pub mod handlers;
pub mod middlewares;
pub mod responses;
"#
    .to_string()
}

pub fn extractors_mod() -> String {
    "// les extractors sont déclarés ici, ex:\n\
     // mod client_context;\n\
     // pub use client_context::ClientContext;\n\
     // -> généré automatiquement via `cargo xtask new extractor <nom>`\n"
        .to_string()
}

pub fn handlers_mod() -> String {
    "pub mod api;\n".to_string()
}

pub fn handlers_api_mod() -> String {
    r#"pub mod health;
pub mod livez;
pub mod v1;
// pub mod readyz;
"#
    .to_string()
}

pub fn health() -> String {
    r#"use axum::{Json, Router, routing::get};
use serde_json::{json, Value};

use crate::config::state::AppState;

// #[utoipa::path(
//     get,
//     path = "/health",
//     responses(
//         (status = 200, description = "Service opérationnel", body = Value,
//          example = json!({"status": "ok"}))
//     ),
//     tag = "health"
// )]
pub async fn health() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health))
}
"#
    .to_string()
}

pub fn livez() -> String {
    r#"use axum::{Json, Router, routing::get};
use serde_json::{json, Value};

use crate::config::state::AppState;

/// Liveness : le process répond. Jamais de dépendance externe ici —
/// un échec déclenche un restart du pod, pas un retrait du load balancer.
// #[utoipa::path(
//     get,
//     path = "/livez",
//     responses(
//         (status = 200, description = "Process vivant", body = Value,
//          example = json!({"status": "alive"}))
//     ),
//     tag = "health"
// )]
pub async fn livez() -> Json<Value> {
    Json(json!({"status": "alive"}))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/livez", get(livez))
}
"#
    .to_string()
}

pub fn readyz() -> String {
    r#"use axum::{Json, Router, extract::State, routing::get};
use serde_json::{json, Value};
use std::time::Duration;

use crate::{config::state::AppState, errors::AppError};

const READYZ_DB_TIMEOUT: Duration = Duration::from_secs(2);
/// Readiness : le service peut traiter du trafic. Vérifie Postgres avec
/// timeout court. 503 → le load balancer retire l'instance du pool.
/// Kafka volontairement exclu : publish fire-and-forget, l'auth reste
/// fonctionnelle si Kafka est down.
// #[utoipa::path(
//     get,
//     path = "/readyz",
//     responses(
//         (status = 200, description = "Prêt à recevoir du trafic", body = Value,
//          example = json!({"status": "ready"})),
//         (status = 503, description = "Dépendance indisponible (Postgres)", body = Value),
//     ),
//     tag = "health"
// )]
pub async fn readyz(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    match tokio::time::timeout(
        READYZ_DB_TIMEOUT,
        sqlx::query("SELECT 1").execute(&state.db),
    )
    .await
    {
        Ok(Ok(_)) => Ok(Json(json!({"status": "ready"}))),
        Ok(Err(e)) => {
            tracing::warn!("readyz: postgres error: {e}");
            Err(AppError::Unavailable("database".into()))
        }
        Err(_) => {
            tracing::warn!("readyz: postgres timeout (> {READYZ_DB_TIMEOUT:?})");
            Err(AppError::Unavailable("database timeout".into()))
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/readyz", get(readyz))
}
"#
    .to_string()
}

pub fn v1_mod() -> String {
    "pub mod metrics;\n".to_string()
}

pub fn v1_metrics_mod() -> String {
    "pub mod prometheus_endpoint;\n".to_string()
}

pub fn v1_metrics_prometheus_endpoint() -> String {
    r#"use axum::{Router, extract::State, http::header, response::IntoResponse, routing::get};

use crate::{
    config::state::AppState,
    errors::AppError,
    http::middlewares::metrics::{is_authorized, render},
};

/// Exposition Prometheus, protégée par METRICS_TOKEN (comparaison en temps
/// constant, cf. clairtyx-metrics).
pub async fn prometheus_endpoint(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    if !is_authorized(
        &headers,
        state.metrics_token_hash.as_ref(),
        state.config.metrics_allow_open,
    ) {
        return Err(AppError::Unauthorized);
    }
    let (content_type, body) = render();
    Ok(([(header::CONTENT_TYPE, content_type)], body))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/metrics", get(prometheus_endpoint))
}
"#
    .to_string()
}

pub fn middlewares_mod() -> String {
    "pub mod metrics;\n".to_string()
}

pub fn middlewares_metrics() -> String {
    r##"//! Instrumentation HTTP Prometheus partagée par les services Clairtyx.
//!
//! Chaque service enregistre SES métriques avec son préfixe
//! (`api_http_requests_total`) — même schéma de
//! labels partout, donc mêmes dashboards/alertes, sans dupliquer le middleware.

use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};
use prometheus::{
    Encoder, HistogramVec, IntCounterVec, IntGauge, TextEncoder, register_histogram_vec,
    register_int_counter_vec, register_int_gauge,
};
use sha2::{Digest, Sha256};
use std::sync::LazyLock;
use std::time::Instant;
use subtle::ConstantTimeEq;

/// Compteurs `api_http_*` — instrumentation partagée clairtyx-metrics,
/// registre global, construits une seule fois.
static METRICS: LazyLock<HttpMetrics> = LazyLock::new(|| HttpMetrics::register("api"));

pub async fn metrics_middleware(request: Request, next: Next) -> Response {
    METRICS.track(request, next).await
}

/// Compteurs HTTP d'un service. À construire UNE fois (LazyLock) : le
/// registre Prometheus est global, un double register paniquerait.
pub struct HttpMetrics {
    requests: IntCounterVec,
    duration: HistogramVec,
    in_flight: IntGauge,
}

impl HttpMetrics {
    /// `prefix` = nom court du service.
    pub fn register(prefix: &str) -> Self {
        Self {
            requests: register_int_counter_vec!(
                format!("{prefix}_http_requests_total"),
                "HTTP requests by method, matched route and status code",
                &["method", "path", "status"]
            )
            .expect("register requests counter"),
            duration: register_histogram_vec!(
                format!("{prefix}_http_request_duration_seconds"),
                "HTTP request latency by method, matched route and status code",
                &["method", "path", "status"],
                vec![
                    0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0
                ]
            )
            .expect("register duration histogram"),
            in_flight: register_int_gauge!(
                format!("{prefix}_http_requests_in_flight"),
                "HTTP requests currently being processed"
            )
            .expect("register in-flight gauge"),
        }
    }

    /// Corps du middleware axum. MatchedPath (template de route, ex:
    /// /me/profil) plutôt que l'URI brute : cardinalité bornée, pas
    /// d'explosion de labels Prometheus.
    pub async fn track(&self, request: Request, next: Next) -> Response {
        tracing::info!(
            "matched={:?} uri={}",
            request
                .extensions()
                .get::<MatchedPath>()
                .map(|p| p.as_str()),
            request.uri()
        );
        let path = request
            .extensions()
            .get::<MatchedPath>()
            .map(|p| p.as_str().to_owned())
            .unwrap_or_else(|| "unmatched".to_owned());
        let method = request.method().to_string();

        self.in_flight.inc();
        let start = Instant::now();
        let response = next.run(request).await;
        self.in_flight.dec();

        let status = response.status().as_u16().to_string();
        self.requests
            .with_label_values(&[&method, &path, &status])
            .inc();
        self.duration
            .with_label_values(&[&method, &path, &status])
            .observe(start.elapsed().as_secs_f64());

        response
    }
}

/// Contrôle d'accès à GET /metrics : Bearer comparé en temps constant au hash
/// SHA-256 attendu. `expected_hash = None` (METRICS_TOKEN absent) → **fail-closed**
/// (refus), sauf dérogation explicite `allow_open` (METRICS_ALLOW_OPEN=true). Un
/// oubli de token n'expose donc jamais les compteurs en silence.
pub fn is_authorized(
    headers: &axum::http::HeaderMap,
    expected_hash: Option<&[u8; 32]>,
    allow_open: bool,
) -> bool {
    let Some(expected) = expected_hash else {
        return allow_open;
    };
    let Some(token) = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    else {
        return false;
    };
    let provided_hash: [u8; 32] = Sha256::digest(token.as_bytes()).into();
    provided_hash.ct_eq(expected).into()
}

/// Sérialise le registre Prometheus global au format texte d'exposition.
pub fn render() -> (&'static str, Vec<u8>) {
    let mut buffer = Vec::new();
    TextEncoder::new()
        .encode(&prometheus::gather(), &mut buffer)
        .unwrap_or_default();
    ("text/plain; version=0.0.4; charset=utf-8", buffer)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue, header};

    fn hash(token: &str) -> [u8; 32] {
        Sha256::digest(token.as_bytes()).into()
    }

    fn bearer(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
        );
        headers
    }

    #[test]
    fn no_token_fails_closed_by_default() {
        // METRICS_TOKEN absent + pas de dérogation → refus.
        assert!(!is_authorized(&HeaderMap::new(), None, false));
    }

    #[test]
    fn no_token_open_only_with_explicit_optin() {
        // METRICS_ALLOW_OPEN=true → ouvert (dev / réseau cloisonné).
        assert!(is_authorized(&HeaderMap::new(), None, true));
    }

    #[test]
    fn correct_token_accepted() {
        let expected = hash("scrape-token");
        assert!(is_authorized(
            &bearer("scrape-token"),
            Some(&expected),
            false
        ));
    }

    #[test]
    fn wrong_or_missing_token_rejected() {
        let expected = hash("scrape-token");
        assert!(!is_authorized(&bearer("autre"), Some(&expected), false));
        assert!(!is_authorized(&HeaderMap::new(), Some(&expected), false));
    }
}
"##
    .to_string()
}

pub fn responses_mod() -> String {
    "// types de réponse HTTP communs (DTOs de sortie, wrappers, ...)\n".to_string()
}
