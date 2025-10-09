use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug, Serialize)]
pub enum ErrorCategory {
    #[serde(rename = "NETWORK_RETRYABLE")]
    NetworkRetryable,
    #[serde(rename = "PERM_CONFIG")]
    PermConfig,
    #[serde(rename = "TRANSIENT_RUNTIME")]
    TransientRuntime,
    #[serde(rename = "PERM_RUNTIME")]
    PermRuntime,
}

#[derive(Debug)]
pub enum CastError {
    DatabaseError(sqlx::Error),
    WasmNotFound(String),
    WasmExecutionFailed(String),
    WasmTimeout,
    InvalidInput(String),
    InternalError(String),
}

impl CastError {
    pub fn category(&self) -> ErrorCategory {
        match self {
            CastError::DatabaseError(_) => ErrorCategory::NetworkRetryable,
            CastError::WasmNotFound(_) => ErrorCategory::PermConfig,
            CastError::WasmExecutionFailed(_) => ErrorCategory::PermRuntime,
            CastError::WasmTimeout => ErrorCategory::TransientRuntime,
            CastError::InvalidInput(_) => ErrorCategory::PermConfig,
            CastError::InternalError(_) => ErrorCategory::NetworkRetryable,
        }
    }

    pub fn error_code(&self) -> &str {
        match self {
            CastError::DatabaseError(_) => "DB_ERROR",
            CastError::WasmNotFound(_) => "WASM_NOT_FOUND",
            CastError::WasmExecutionFailed(_) => "WASM_EXEC_FAILED",
            CastError::WasmTimeout => "WASM_TIMEOUT",
            CastError::InvalidInput(_) => "INVALID_INPUT",
            CastError::InternalError(_) => "INTERNAL_ERROR",
        }
    }
}

impl fmt::Display for CastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CastError::DatabaseError(e) => write!(f, "Database error: {}", e),
            CastError::WasmNotFound(name) => write!(f, "WASM module not found: {}", name),
            CastError::WasmExecutionFailed(msg) => write!(f, "WASM execution failed: {}", msg),
            CastError::WasmTimeout => write!(f, "WASM execution timeout"),
            CastError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            CastError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl ResponseError for CastError {
    fn status_code(&self) -> StatusCode {
        match self {
            CastError::DatabaseError(_) => StatusCode::SERVICE_UNAVAILABLE,
            CastError::WasmNotFound(_) => StatusCode::NOT_FOUND,
            CastError::WasmExecutionFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CastError::WasmTimeout => StatusCode::REQUEST_TIMEOUT,
            CastError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            CastError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        #[derive(Serialize)]
        struct ErrorResponse {
            error: String,
            error_code: String,
            category: ErrorCategory,
        }

        HttpResponse::build(self.status_code()).json(ErrorResponse {
            error: self.to_string(),
            error_code: self.error_code().to_string(),
            category: self.category(),
        })
    }
}

impl From<sqlx::Error> for CastError {
    fn from(err: sqlx::Error) -> Self {
        CastError::DatabaseError(err)
    }
}
