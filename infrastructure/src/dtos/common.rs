//! Common DTOs and error responses

use actix_web::{HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Standard error response following RFC 7807 (Problem Details for HTTP APIs)
///
/// Provides a consistent error format across all API endpoints.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// HTTP status code
    pub status: u16,

    /// Error type (e.g., "validation_error", "not_found")
    pub error: String,

    /// Human-readable error message
    pub message: String,

    /// Optional detailed error information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<String>>,
}

impl ErrorResponse {
    pub fn new(status: u16, error: &str, message: &str) -> Self {
        Self {
            status,
            error: error.to_string(),
            message: message.to_string(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: Vec<String>) -> Self {
        self.details = Some(details);
        self
    }

    pub fn bad_request(message: &str) -> Self {
        Self::new(400, "bad_request", message)
    }

    pub fn unauthorized(message: &str) -> Self {
        Self::new(401, "unauthorized", message)
    }

    pub fn forbidden(message: &str) -> Self {
        Self::new(403, "forbidden", message)
    }

    pub fn not_found(message: &str) -> Self {
        Self::new(404, "not_found", message)
    }

    pub fn conflict(message: &str) -> Self {
        Self::new(409, "conflict", message)
    }

    pub fn internal_error(message: &str) -> Self {
        Self::new(500, "internal_error", message)
    }
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error, self.message)
    }
}

impl ResponseError for ErrorResponse {
    fn error_response(&self) -> HttpResponse {
        match self.status {
            400 => HttpResponse::BadRequest().json(self),
            401 => HttpResponse::Unauthorized().json(self),
            403 => HttpResponse::Forbidden().json(self),
            404 => HttpResponse::NotFound().json(self),
            409 => HttpResponse::Conflict().json(self),
            500 => HttpResponse::InternalServerError().json(self),
            _ => HttpResponse::InternalServerError().json(self),
        }
    }
}

/// Success response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub data: T,
}

impl<T> SuccessResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            success: true,
            data,
        }
    }
}
