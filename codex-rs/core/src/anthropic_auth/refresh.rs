//! Proactive and forced refresh of Anthropic OAuth tokens.
//!
//! These free functions delegate to the Anthropic SDK for the actual token
//! exchange and then persist the result via the auth storage layer. The
//! [`AuthManager`] methods are thin wrappers around these.

use crate::auth::AuthManager;
use crate::auth::CodexAuth;
use crate::auth::ProviderAuth;
use crate::auth::ProviderName;
use crate::auth::RefreshTokenError;
use crate::auth::create_auth_storage;
use crate::error::RefreshTokenFailedError;
use crate::error::RefreshTokenFailedReason;

use super::types::ANTHROPIC_TOKEN_REFRESH_BUFFER_SECONDS;

/// Proactively refresh an Anthropic OAuth token if it is expiring soon.
///
/// Best-effort: failures are logged but do not prevent the request from
/// proceeding (the stale token might still work, and 401 recovery handles
/// the rest).
pub(crate) async fn refresh_if_needed(manager: &AuthManager) {
    let auth = match manager.auth_cached_for_provider(ProviderName::Anthropic) {
        Some(CodexAuth::AnthropicOAuth(ref oauth)) => oauth.clone(),
        _ => return,
    };

    if !auth.is_expiring_within(ANTHROPIC_TOKEN_REFRESH_BUFFER_SECONDS) {
        return;
    }

    tracing::info!("Anthropic OAuth token expiring soon, refreshing proactively");
    let client = crate::default_client::build_reqwest_client();
    match orbit_code_anthropic::refresh_anthropic_token(&client, auth.refresh_token()).await {
        Ok(tokens) => {
            let now = chrono::Utc::now().timestamp();
            let expires_at = now.saturating_add(i64::try_from(tokens.expires_in).unwrap_or(3600));
            // Persist refreshed tokens. ONLY reload cache after a successful save.
            let storage = create_auth_storage(
                manager.orbit_code_home().to_path_buf(),
                manager.auth_credentials_store_mode(),
            );
            match storage.load() {
                Ok(Some(mut v2)) => {
                    v2.set_provider_auth(
                        ProviderName::Anthropic,
                        ProviderAuth::AnthropicOAuth {
                            access_token: tokens.access_token,
                            refresh_token: tokens.refresh_token,
                            expires_at,
                        },
                    );
                    match storage.save(&v2) {
                        Ok(()) => {
                            manager.reload();
                        }
                        Err(e) => {
                            tracing::warn!("Anthropic refresh succeeded but persist failed: {e}")
                        }
                    }
                }
                Ok(None) => {
                    tracing::warn!(
                        "Anthropic refresh succeeded but no auth storage found; keeping cached token"
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "Anthropic refresh succeeded but storage unreadable: {e}; keeping cached token"
                    );
                }
            }
        }
        Err(e) => {
            tracing::warn!("Proactive Anthropic OAuth refresh failed: {e}");
        }
    }
}

/// Force-refresh an Anthropic OAuth token. Unlike proactive refresh, this
/// always refreshes (no expiry check) and returns a typed error.
pub(crate) async fn force_refresh(
    manager: &AuthManager,
) -> std::result::Result<(), RefreshTokenError> {
    let auth = manager.auth_cached_for_provider(ProviderName::Anthropic);
    let Some(CodexAuth::AnthropicOAuth(ref oauth)) = auth else {
        return Err(RefreshTokenError::Permanent(RefreshTokenFailedError::new(
            RefreshTokenFailedReason::Other,
            "not AnthropicOAuth",
        )));
    };
    let client = crate::default_client::build_reqwest_client();
    let tokens = orbit_code_anthropic::refresh_anthropic_token(&client, oauth.refresh_token())
        .await
        .map_err(|e| RefreshTokenError::Transient(std::io::Error::other(e.to_string())))?;

    let now = chrono::Utc::now().timestamp();
    let expires_at = now.saturating_add(i64::try_from(tokens.expires_in).unwrap_or(3600));
    let storage = create_auth_storage(
        manager.orbit_code_home().to_path_buf(),
        manager.auth_credentials_store_mode(),
    );
    match storage.load() {
        Ok(Some(mut v2)) => {
            v2.set_provider_auth(
                ProviderName::Anthropic,
                ProviderAuth::AnthropicOAuth {
                    access_token: tokens.access_token,
                    refresh_token: tokens.refresh_token,
                    expires_at,
                },
            );
            storage.save(&v2).map_err(RefreshTokenError::Transient)?;
            manager.reload();
            Ok(())
        }
        Ok(None) => {
            tracing::warn!("Force refresh succeeded but no auth storage; keeping cached token");
            Err(RefreshTokenError::Transient(std::io::Error::other(
                "no auth storage after refresh",
            )))
        }
        Err(e) => {
            tracing::warn!("Force refresh succeeded but storage unreadable: {e}");
            Err(RefreshTokenError::Transient(e))
        }
    }
}
