//! Anthropic-specific authentication types and token refresh logic.

/// Anthropic API key auth (persisted in auth storage).
#[derive(Debug, Clone)]
pub struct AnthropicApiKeyAuth {
    pub(crate) api_key: String,
}

impl AnthropicApiKeyAuth {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

/// Anthropic OAuth auth (code-paste flow, refreshable).
#[derive(Debug, Clone)]
pub struct AnthropicOAuthAuth {
    pub(crate) access_token: String,
    pub(crate) refresh_token: String,
    /// Unix timestamp in seconds when access_token expires.
    pub(crate) expires_at: i64,
}

impl AnthropicOAuthAuth {
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
    }

    /// Returns true if the access token will expire within the given buffer (seconds).
    pub fn is_expiring_within(&self, buffer_seconds: i64) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.expires_at.saturating_sub(now) < buffer_seconds
    }
}

/// Buffer in seconds before expiry to trigger proactive refresh.
pub const ANTHROPIC_TOKEN_REFRESH_BUFFER_SECONDS: i64 = 300; // 5 minutes
