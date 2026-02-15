//! Error types for the SpareBank 1 API client.

use thiserror::Error;

/// Errors that can occur when interacting with the SpareBank 1 API.
#[derive(Debug, Error)]
pub enum ApiError {
    /// HTTP request failed
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// API returned an error response
    #[error("API error: {code} - {message}")]
    Api {
        code: String,
        message: String,
        trace_id: String,
    },

    /// Failed to parse response
    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),

    /// Configuration error
    #[error("Config error: {0}")]
    Config(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Token not available
    #[error("No access token available")]
    NoToken,
}
