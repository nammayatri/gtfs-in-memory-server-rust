use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use tracing::{error, warn};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("HTTP request failed: {0}")]
    HttpRequest(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    DbError(String),

    #[error("Service not ready: {0}")]
    NotReady(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Data processing error: {0}")]
    DataProcessing(String),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::HttpRequest(_) => StatusCode::BAD_GATEWAY,
            AppError::DbError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotReady(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::RateLimit(_) => StatusCode::TOO_MANY_REQUESTS,
            AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DataProcessing(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::HttpRequest(_) => "HTTP_REQUEST_ERROR",
            AppError::DbError(_) => "DATABASE_ERROR",
            AppError::NotReady(_) => "SERVICE_NOT_READY",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::RateLimit(_) => "RATE_LIMIT_EXCEEDED",
            AppError::Config(_) => "CONFIGURATION_ERROR",
            AppError::DataProcessing(_) => "DATA_PROCESSING_ERROR",
        }
    }

    pub fn log_error(&self, context: &str) {
        match self {
            AppError::NotFound(msg) => {
                warn!("Not found error in {}: {}", context, msg);
            }
            AppError::Internal(msg) => {
                error!("Internal error in {}: {}", context, msg);
            }
            AppError::HttpRequest(err) => {
                error!("HTTP request error in {}: {}", context, err);
            }
            AppError::DbError(msg) => {
                error!("Database error in {}: {}", context, msg);
            }
            AppError::NotReady(msg) => {
                warn!("Service not ready in {}: {}", context, msg);
            }
            AppError::Validation(msg) => {
                warn!("Validation error in {}: {}", context, msg);
            }
            AppError::RateLimit(msg) => {
                warn!("Rate limit exceeded in {}: {}", context, msg);
            }
            AppError::Config(msg) => {
                error!("Configuration error in {}: {}", context, msg);
            }
            AppError::DataProcessing(msg) => {
                error!("Data processing error in {}: {}", context, msg);
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Log the error with context
        self.log_error("HTTP response");

        let status_code = self.status_code();
        let error_code = self.error_code();
        let message = self.to_string();

        let error_response = json!({
            "error": {
                "code": error_code,
                "message": message,
                "status": status_code.as_u16(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }
        });

        (status_code, Json(error_response)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

// Helper functions for creating errors with context
pub fn not_found_error(resource: &str, identifier: &str) -> AppError {
    AppError::NotFound(format!(
        "{} not found with identifier: {}",
        resource, identifier
    ))
}

pub fn data_processing_error(operation: &str, details: &str) -> AppError {
    AppError::DataProcessing(format!("Failed to {}: {}", operation, details))
}
