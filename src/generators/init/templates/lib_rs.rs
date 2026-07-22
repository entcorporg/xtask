pub fn content() -> String {
    r#"//! Core library pour le backend Rust.
//! Expose les domaines métier, la couche HTTP, l'infrastructure et la configuration.

pub mod bootstrap;
pub mod config;
pub mod domains;
pub mod http;
pub mod infrastructure;
pub mod routes;

// Export réutilisable d'erreurs globales au niveau de la crate
pub mod errors {
    use axum::{
        Json,
        http::StatusCode,
        response::{IntoResponse, Response},
    };
    use serde_json::json;

    /// Type de résultat globalisé pour l'application
    pub type AppResult<T> = Result<T, AppError>;

    /// Enum globale d'erreur HTTP mappable en réponse JSON
    #[derive(Debug, thiserror::Error)]
    pub enum AppError {
        #[error("Ressource non trouvée")]
        NotFound,

        #[error("Erreur de validation: {0}")]
        ValidationError(String),

        #[error("Erreur interne du serveur")]
        Internal(#[from] anyhow::Error),

        #[error("Non autorisé")]
        Unauthorized,
    }

    impl IntoResponse for AppError {
        fn into_response(self) -> Response {
            let (status, message) = match self {
                AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
                AppError::ValidationError(ref msg) => (StatusCode::BAD_REQUEST, msg.clone()),
                AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
                AppError::Internal(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Erreur interne du serveur".to_string(),
                ),
            };

            let body = Json(json!({
                "error": {
                    "message": message,
                    "code": status.as_u16()
                }
            }));

            (status, body).into_response()
        }
    }
}
"#
    .to_string()
}
