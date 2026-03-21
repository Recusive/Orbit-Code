//! Anthropic OAuth login flow for Orbit Code.
//!
//! This module implements the browser-based OAuth authorization code flow for
//! authenticating with Anthropic's API. It supports two authentication modes:
//!
//! - **Max subscription** users who authenticate via `console.anthropic.com`
//! - **Console API key** users who create a permanent API key after OAuth
//!
//! The flow uses PKCE (S256) and a code-paste callback mode where the user
//! copies a `{code}#{state}` string back into the CLI.

use serde::Deserialize;
use std::time::Duration;
use thiserror::Error;
use url::Url;

use crate::pkce::generate_pkce;

/// Anthropic OAuth client ID for the CLI application.
const ANTHROPIC_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";

/// Token endpoint for exchanging authorization codes and refreshing tokens.
const ANTHROPIC_TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";

/// Redirect URI configured for the code-paste callback mode.
const ANTHROPIC_REDIRECT_URI: &str = "https://console.anthropic.com/oauth/code/callback";

/// OAuth scopes requested during authorization.
const ANTHROPIC_SCOPES: &str = "org:create_api_key user:profile user:inference";

/// Endpoint for creating a permanent API key from an OAuth access token.
const ANTHROPIC_CREATE_API_KEY_URL: &str =
    "https://api.anthropic.com/api/oauth/claude_cli/create_api_key";

/// Authorization base URL for Claude Pro/Max OAuth (tokens used directly).
const ANTHROPIC_AUTHORIZE_URL_MAX: &str = "https://claude.ai/oauth/authorize";

/// Authorization base URL for Console API Key OAuth (exchanges for permanent key).
const ANTHROPIC_AUTHORIZE_URL_CONSOLE: &str = "https://console.anthropic.com/oauth/authorize";

/// Determines which Anthropic authentication path to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnthropicAuthMode {
    /// Max subscription user -- uses OAuth tokens directly for inference.
    MaxSubscription,
    /// Console API key user -- exchanges OAuth tokens for a permanent API key.
    ConsoleApiKey,
}

/// Tokens returned by the Anthropic OAuth token endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicTokens {
    /// Bearer access token for API calls.
    pub access_token: String,
    /// Refresh token for obtaining new access tokens.
    pub refresh_token: String,
    /// Lifetime of the access token in seconds.
    pub expires_in: u64,
}

/// Errors that can occur during the Anthropic OAuth flow.
#[derive(Debug, Error)]
pub enum AnthropicLoginError {
    /// An HTTP transport or network error occurred.
    #[error(transparent)]
    Http(#[from] reqwest::Error),

    /// The token endpoint returned a non-success status.
    #[error("token exchange failed: {0}")]
    TokenExchange(String),

    /// Creating a permanent API key failed.
    #[error("API key creation failed: {0}")]
    ApiKeyCreation(String),

    /// The pasted code string was not in the expected `{code}#{state}` format.
    #[error("invalid code format: expected {{code}}#{{state}}")]
    InvalidCode,

    /// Failed to build the HTTP client (e.g., CA certificate error).
    #[error("Failed to build HTTP client: {0}")]
    ClientBuild(String),

    /// Authorization code is from a different or expired login attempt.
    #[error(
        "Authorization code is from a different or expired login attempt. Please restart the sign-in flow."
    )]
    StaleAttempt,
}

/// Builds a CA-aware reqwest client with connect and request timeouts.
fn build_anthropic_login_client() -> Result<reqwest::Client, AnthropicLoginError> {
    orbit_code_client::build_reqwest_client_with_custom_ca(
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30)),
    )
    .map_err(|e| AnthropicLoginError::ClientBuild(e.to_string()))
}

/// Generates an Anthropic authorization URL and the PKCE verifier.
///
/// The returned tuple is `(authorize_url, pkce_verifier)`. The caller should
/// open the URL in a browser and prompt the user to paste the resulting
/// `{code}#{state}` string.
pub fn anthropic_authorize_url(
    mode: AnthropicAuthMode,
) -> Result<(String, String), AnthropicLoginError> {
    let pkce = generate_pkce();

    // State is set to the PKCE verifier so it can be verified when the code
    // is pasted back.
    let state = &pkce.code_verifier;

    let base_url = match mode {
        AnthropicAuthMode::MaxSubscription => ANTHROPIC_AUTHORIZE_URL_MAX,
        AnthropicAuthMode::ConsoleApiKey => ANTHROPIC_AUTHORIZE_URL_CONSOLE,
    };
    let mut url = Url::parse(base_url)
        .map_err(|e| AnthropicLoginError::ClientBuild(format!("invalid authorize URL: {e}")))?;

    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", ANTHROPIC_CLIENT_ID)
        .append_pair("redirect_uri", ANTHROPIC_REDIRECT_URI)
        .append_pair("scope", ANTHROPIC_SCOPES)
        .append_pair("code_challenge", &pkce.code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("state", state)
        .append_pair("code", "true");

    Ok((url.to_string(), pkce.code_verifier))
}

/// Exchanges a pasted `{code}#{state}` string for Anthropic OAuth tokens.
///
/// The `code_with_state` parameter is in the format `{code}#{state}` as
/// returned by the Anthropic OAuth code-paste callback. The `verifier` is
/// the PKCE code verifier returned by [`anthropic_authorize_url`].
pub async fn anthropic_exchange_code(
    code_with_state: &str,
    verifier: &str,
) -> Result<AnthropicTokens, AnthropicLoginError> {
    let (code, pasted_state) = code_with_state
        .rsplit_once('#')
        .ok_or(AnthropicLoginError::InvalidCode)?;

    if code.is_empty() {
        return Err(AnthropicLoginError::InvalidCode);
    }

    // Validate pasted state matches the PKCE verifier from this attempt.
    if pasted_state != verifier {
        tracing::warn!("anthropic_oauth_stale_attempt");
        return Err(AnthropicLoginError::StaleAttempt);
    }

    let body = serde_json::json!({
        "grant_type": "authorization_code",
        "client_id": ANTHROPIC_CLIENT_ID,
        "code": code,
        "state": pasted_state,
        "redirect_uri": ANTHROPIC_REDIRECT_URI,
        "code_verifier": verifier,
    });

    let client = build_anthropic_login_client()?;
    let resp = client
        .post(ANTHROPIC_TOKEN_URL)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AnthropicLoginError::TokenExchange(format!(
            "status {status}: {text}"
        )));
    }

    let tokens: AnthropicTokens = resp.json().await?;
    Ok(tokens)
}

/// Refreshes an Anthropic OAuth access token using a refresh token.
pub async fn anthropic_refresh_token(
    refresh_token: &str,
) -> Result<AnthropicTokens, AnthropicLoginError> {
    let body = serde_json::json!({
        "grant_type": "refresh_token",
        "client_id": ANTHROPIC_CLIENT_ID,
        "refresh_token": refresh_token,
    });

    let client = build_anthropic_login_client()?;
    let resp = client
        .post(ANTHROPIC_TOKEN_URL)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AnthropicLoginError::TokenExchange(format!(
            "status {status}: {text}"
        )));
    }

    let tokens: AnthropicTokens = resp.json().await?;
    Ok(tokens)
}

/// Creates a permanent Anthropic API key using an OAuth access token.
///
/// Returns the raw API key string on success. The key can be stored and used
/// for subsequent API calls without needing to refresh OAuth tokens.
pub async fn anthropic_create_api_key(access_token: &str) -> Result<String, AnthropicLoginError> {
    #[derive(Deserialize)]
    struct CreateApiKeyResponse {
        raw_key: String,
    }

    let client = build_anthropic_login_client()?;
    let resp = client
        .post(ANTHROPIC_CREATE_API_KEY_URL)
        .header("Authorization", format!("Bearer {access_token}"))
        .header("Content-Length", "0")
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AnthropicLoginError::ApiKeyCreation(format!(
            "status {status}: {text}"
        )));
    }

    let body: CreateApiKeyResponse = resp.json().await?;
    Ok(body.raw_key)
}
