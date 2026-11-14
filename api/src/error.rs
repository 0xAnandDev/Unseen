//! One error type for the whole API, turned into a JSON response
//! like `{ "error": "..." }` with the right HTTP status code.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// The client sent something invalid (HTTP 400).
    #[error("{0}")]
    BadRequest(String),

    /// The thing asked for does not exist, or the key was wrong (HTTP 404).
    #[error("{0}")]
    NotFound(String),

    /// Something failed in the database (HTTP 500).
    #[error("database error")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Database(e) => {
                // Log the real cause on the server, but don't leak it to clients.
                tracing::error!(error = %e, "database error");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}
