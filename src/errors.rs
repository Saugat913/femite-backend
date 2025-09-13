use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Image upload error: {0}")]
    ImageUpload(String),
    
    #[error("File too large: maximum size is {max_size} bytes, got {actual_size} bytes")]
    FileTooLarge { max_size: u64, actual_size: u64 },
    
    #[error("Invalid file type: expected {expected}, got {actual}")]
    InvalidFileType { expected: String, actual: String },
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref err) => {
                tracing::error!("Database error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred")
            }
            AppError::ImageUpload(ref msg) => {
                tracing::error!("Image upload error: {}", msg);
                (StatusCode::BAD_REQUEST, "Image upload failed")
            }
            AppError::FileTooLarge { max_size, actual_size } => {
                tracing::warn!("File too large: {} > {}", actual_size, max_size);
                (StatusCode::PAYLOAD_TOO_LARGE, "File size exceeds maximum limit")
            }
            AppError::InvalidFileType { ref expected, ref actual } => {
                tracing::warn!("Invalid file type: expected {}, got {}", expected, actual);
                (StatusCode::BAD_REQUEST, "Invalid file type")
            }
            AppError::Validation(ref msg) => {
                tracing::warn!("Validation error: {}", msg);
                (StatusCode::BAD_REQUEST, "Validation failed")
            }
            AppError::NotFound(ref msg) => {
                tracing::info!("Resource not found: {}", msg);
                (StatusCode::NOT_FOUND, "Resource not found")
            }
            AppError::Unauthorized => {
                tracing::warn!("Unauthorized access attempt");
                (StatusCode::UNAUTHORIZED, "Unauthorized")
            }
            AppError::Forbidden(ref msg) => {
                tracing::warn!("Forbidden access: {}", msg);
                (StatusCode::FORBIDDEN, "Forbidden")
            }
            AppError::Internal(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        let details = self.to_string();
        let body = Json(json!({
            "error": error_message,
            "details": details
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
