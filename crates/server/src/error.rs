//! Erreurs HTTP : même contrat que FastAPI (`{"detail": "..."}`), messages en français.

use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_extra::extract::QueryRejection;
use serde_json::json;
use tdb_core::Erreur;

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub detail: String,
}

impl ApiError {
    pub fn new(status: StatusCode, detail: impl Into<String>) -> Self {
        ApiError {
            status,
            detail: detail.into(),
        }
    }
    pub fn non_authentifie(detail: &str) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, detail)
    }
    pub fn requete(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, detail)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "detail": self.detail }))).into_response()
    }
}

impl From<Erreur> for ApiError {
    fn from(e: Erreur) -> Self {
        match e {
            Erreur::Validation(m) => ApiError::new(StatusCode::BAD_REQUEST, m),
            Erreur::Introuvable(m) => ApiError::new(StatusCode::NOT_FOUND, m),
            Erreur::Reseau(m) => ApiError::new(StatusCode::BAD_GATEWAY, m),
            // Détails internes (SQL, verrous, secrets) : journalisés, jamais renvoyés au client.
            autre => {
                tracing::error!("Erreur interne : {autre}");
                ApiError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Erreur interne du serveur.",
                )
            }
        }
    }
}

impl From<String> for ApiError {
    fn from(message: String) -> Self {
        ApiError::requete(message)
    }
}

impl From<JsonRejection> for ApiError {
    fn from(r: JsonRejection) -> Self {
        let status = if r.status() == StatusCode::PAYLOAD_TOO_LARGE {
            StatusCode::PAYLOAD_TOO_LARGE
        } else {
            StatusCode::UNPROCESSABLE_ENTITY
        };
        ApiError::new(status, "Données de la requête invalides.")
    }
}

impl From<QueryRejection> for ApiError {
    fn from(_: QueryRejection) -> Self {
        ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, "Paramètres invalides.")
    }
}

impl From<PathRejection> for ApiError {
    fn from(_: PathRejection) -> Self {
        ApiError::new(StatusCode::UNPROCESSABLE_ENTITY, "Identifiant invalide.")
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
