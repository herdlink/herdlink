use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

pub struct AppError(pub StatusCode, pub &'static str);

impl AppError {
    pub fn bad_request(message: &'static str) -> Self {
        Self(StatusCode::BAD_REQUEST, message)
    }
    pub fn unauthorized() -> Self {
        Self(StatusCode::UNAUTHORIZED, "authentication required")
    }
    pub fn forbidden() -> Self {
        Self(StatusCode::FORBIDDEN, "insufficient permissions")
    }
    pub fn not_found() -> Self {
        Self(StatusCode::NOT_FOUND, "resource not found")
    }
    pub fn internal(error: impl std::fmt::Display) -> Self {
        tracing::error!(%error, "request failed");
        Self(StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error": self.1}))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        if let sqlx::Error::Database(ref db) = error {
            if db.is_unique_violation() {
                return Self(StatusCode::CONFLICT, "resource already exists");
            }
            if db.is_foreign_key_violation() {
                return Self::bad_request(
                    "referenced resource does not exist or belongs to another scope",
                );
            }
            if db.is_check_violation() {
                return Self::bad_request("invalid field value");
            }
        }
        if matches!(error, sqlx::Error::RowNotFound) {
            return Self::not_found();
        }
        Self::internal(error)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
