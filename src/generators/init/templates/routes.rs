pub fn content() -> String {
    r#"use axum::{Router, middleware};

use crate::{
    config::state::AppState,
    http::{handlers::api, middlewares::metrics::metrics_middleware},
};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(api::v1::metrics::prometheus_endpoint::router())
        .merge(api::health::router())
        .merge(api::livez::router())
        .layer(middleware::from_fn(metrics_middleware))
        .with_state(state)
}
"#
    .to_string()
}
