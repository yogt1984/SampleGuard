use crate::error::SampleGuardError;
use actix_web::{HttpResponse, ResponseError};
use serde_json::json;

/// API-specific error type
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("SampleGuard error: {0}")]
    SampleGuard(#[from] SampleGuardError),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::SampleGuard(e) => {
                HttpResponse::InternalServerError().json(json!({
                    "error": "SampleGuard error",
                    "message": e.to_string()
                }))
            }
            ApiError::Validation(msg) => {
                HttpResponse::BadRequest().json(json!({
                    "error": "Validation error",
                    "message": msg
                }))
            }
            ApiError::NotFound(msg) => {
                HttpResponse::NotFound().json(json!({
                    "error": "Not found",
                    "message": msg
                }))
            }
            ApiError::Internal(msg) => {
                HttpResponse::InternalServerError().json(json!({
                    "error": "Internal server error",
                    "message": msg
                }))
            }
        }
    }
}

