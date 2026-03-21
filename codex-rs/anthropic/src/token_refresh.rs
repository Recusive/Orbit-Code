//! Anthropic OAuth token refresh — minimal HTTP client for token endpoint.

use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;

const ANTHROPIC_TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";
const ANTHROPIC_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";

/// Tokens returned from a successful refresh.
#[derive(Debug, Deserialize)]
pub struct RefreshedTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

/// Refresh an Anthropic OAuth access token using a refresh token.
/// Uses JSON body (not form-encoded), matching the Anthropic token endpoint spec.
pub async fn refresh_anthropic_token(
    client: &Client,
    refresh_token: &str,
) -> Result<RefreshedTokens, crate::AnthropicError> {
    #[derive(Serialize)]
    struct RefreshRequest<'a> {
        grant_type: &'static str,
        refresh_token: &'a str,
        client_id: &'static str,
    }

    let resp = client
        .post(ANTHROPIC_TOKEN_URL)
        .json(&RefreshRequest {
            grant_type: "refresh_token",
            refresh_token,
            client_id: ANTHROPIC_CLIENT_ID,
        })
        .send()
        .await
        .map_err(|e| {
            crate::AnthropicError::Transport(orbit_code_client::TransportError::Network(
                e.to_string(),
            ))
        })?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.ok();
        return Err(crate::AnthropicError::Api(Box::new(
            crate::AnthropicApiError {
                status,
                error_type: "token_refresh_failed".to_string(),
                message: body.unwrap_or_else(|| "refresh failed".to_string()),
            },
        )));
    }

    resp.json::<RefreshedTokens>().await.map_err(|e| {
        crate::AnthropicError::Transport(orbit_code_client::TransportError::Network(format!(
            "failed to decode refresh response: {e}"
        )))
    })
}
