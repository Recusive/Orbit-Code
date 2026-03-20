//! Error types for the Anthropic Messages API client.

use orbit_code_client::TransportError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AnthropicError>;

#[derive(Debug)]
pub struct AnthropicApiError {
    pub status: u16,
    pub error_type: String,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum AnthropicError {
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error(
        "Anthropic API error ({status}): [{error_type}] {message}",
        status = .0.status,
        error_type = .0.error_type,
        message = .0.message
    )]
    Api(Box<AnthropicApiError>),
    #[error("SSE parse error: {0}")]
    StreamParse(String),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("Anthropic overloaded (529)")]
    Overloaded,
    #[error("Rate limited (429)")]
    RateLimited,
}

impl AnthropicError {
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, Self::RateLimited)
    }

    pub fn is_overloaded(&self) -> bool {
        matches!(self, Self::Overloaded)
    }
}
