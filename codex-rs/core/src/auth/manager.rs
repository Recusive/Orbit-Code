//! Central authentication manager.
//!
//! [`AuthManager`] is the single source of truth for the current session's
//! authentication state. It loads credentials once (or on explicit reload),
//! caches them behind a `RwLock`, and hands out cloned [`CodexAuth`] values so
//! the rest of the program sees a consistent snapshot.

use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

use chrono::Utc;
use orbit_code_app_server_protocol::AuthMode as ApiAuthMode;

use crate::anthropic_auth::AnthropicApiKeyAuth;
use crate::auth::AuthDotJson;
use crate::auth::AuthMode;
use crate::auth::CLIENT_ID;
use crate::auth::ChatgptAuth;
use crate::auth::CodexAuth;
use crate::auth::ExternalAuthRefreshContext;
use crate::auth::ExternalAuthRefreshReason;
use crate::auth::ExternalAuthRefresher;
use crate::auth::REFRESH_TOKEN_ACCOUNT_MISMATCH_MESSAGE;
use crate::auth::REFRESH_TOKEN_EXPIRED_MESSAGE;
use crate::auth::REFRESH_TOKEN_INVALIDATED_MESSAGE;
use crate::auth::REFRESH_TOKEN_REUSED_MESSAGE;
use crate::auth::REFRESH_TOKEN_UNKNOWN_MESSAGE;
use crate::auth::REFRESH_TOKEN_URL;
use crate::auth::REFRESH_TOKEN_URL_OVERRIDE_ENV_VAR;
use crate::auth::RefreshTokenError;
use crate::auth::TOKEN_REFRESH_INTERVAL;
use crate::auth::persistence::codex_auth_from_provider_auth;
use crate::auth::persistence::load_auth;
use crate::auth::persistence::load_auth_dot_json_v2;
use crate::auth::persistence::logout_all_stores;
use crate::auth::persistence::persist_tokens;
use crate::auth::persistence::save_auth;
use crate::auth::recovery::UnauthorizedRecovery;
use crate::auth::storage::AuthCredentialsStoreMode;
use crate::auth::storage::ProviderName;
use crate::error::RefreshTokenFailedError;
use crate::error::RefreshTokenFailedReason;
use crate::util::try_parse_error_message;
use orbit_code_client::CodexHttpClient;
use reqwest::StatusCode;
use serde::Deserialize;
use serde::Serialize;

/// Internal cached auth state.
#[derive(Clone)]
struct CachedAuth {
    auth: Option<CodexAuth>,
    /// Callback used to refresh external auth by asking the parent app for new tokens.
    external_refresher: Option<Arc<dyn ExternalAuthRefresher>>,
}

impl Debug for CachedAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedAuth")
            .field(
                "auth_mode",
                &self.auth.as_ref().map(CodexAuth::api_auth_mode),
            )
            .field(
                "external_refresher",
                &self.external_refresher.as_ref().map(|_| "present"),
            )
            .finish()
    }
}

pub(super) enum ReloadOutcome {
    /// Reload was performed and the cached auth changed
    ReloadedChanged,
    /// Reload was performed and the cached auth remained the same
    ReloadedNoChange,
    /// Reload was skipped (missing or mismatched account id)
    Skipped,
}

/// Central manager providing a single source of truth for auth.json derived
/// authentication data. It loads once (or on preference change) and then
/// hands out cloned `CodexAuth` values so the rest of the program has a
/// consistent snapshot.
///
/// External modifications to `auth.json` will NOT be observed until
/// `reload()` is called explicitly. This matches the design goal of avoiding
/// different parts of the program seeing inconsistent auth data mid‑run.
#[derive(Debug)]
pub struct AuthManager {
    orbit_code_home: PathBuf,
    inner: RwLock<CachedAuth>,
    enable_orbit_code_api_key_env: bool,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
    forced_chatgpt_workspace_id: RwLock<Option<String>>,
}

impl AuthManager {
    /// Create a new manager loading the initial auth using the provided
    /// preferred auth method. Errors loading auth are swallowed; `auth()` will
    /// simply return `None` in that case so callers can treat it as an
    /// unauthenticated state.
    pub fn new(
        orbit_code_home: PathBuf,
        enable_orbit_code_api_key_env: bool,
        auth_credentials_store_mode: AuthCredentialsStoreMode,
    ) -> Self {
        let managed_auth = load_auth(
            &orbit_code_home,
            enable_orbit_code_api_key_env,
            auth_credentials_store_mode,
        )
        .ok()
        .flatten();
        Self {
            orbit_code_home,
            inner: RwLock::new(CachedAuth {
                auth: managed_auth,
                external_refresher: None,
            }),
            enable_orbit_code_api_key_env,
            auth_credentials_store_mode,
            forced_chatgpt_workspace_id: RwLock::new(None),
        }
    }

    /// Create an AuthManager with a specific CodexAuth, for testing only.
    pub(crate) fn from_auth_for_testing(auth: CodexAuth) -> Arc<Self> {
        let cached = CachedAuth {
            auth: Some(auth),
            external_refresher: None,
        };

        Arc::new(Self {
            orbit_code_home: PathBuf::from("non-existent"),
            inner: RwLock::new(cached),
            enable_orbit_code_api_key_env: false,
            auth_credentials_store_mode: AuthCredentialsStoreMode::File,
            forced_chatgpt_workspace_id: RwLock::new(None),
        })
    }

    /// Create an AuthManager with a specific CodexAuth and codex home, for testing only.
    pub(crate) fn from_auth_for_testing_with_home(
        auth: CodexAuth,
        orbit_code_home: PathBuf,
    ) -> Arc<Self> {
        let cached = CachedAuth {
            auth: Some(auth),
            external_refresher: None,
        };
        Arc::new(Self {
            orbit_code_home,
            inner: RwLock::new(cached),
            enable_orbit_code_api_key_env: false,
            auth_credentials_store_mode: AuthCredentialsStoreMode::File,
            forced_chatgpt_workspace_id: RwLock::new(None),
        })
    }

    /// Current cached auth (clone) without attempting a refresh.
    pub fn auth_cached(&self) -> Option<CodexAuth> {
        self.inner.read().ok().and_then(|c| c.auth.clone())
    }

    /// Get cached auth for a specific provider without refresh.
    ///
    /// Checks the currently cached auth and returns it only if it belongs
    /// to the requested provider. Also checks env vars and persistent storage
    /// for the requested provider if the cached auth doesn't match.
    pub fn auth_cached_for_provider(&self, provider: ProviderName) -> Option<CodexAuth> {
        // Check if cached auth matches the requested provider
        if let Some(auth) = self.auth_cached() {
            let is_match = matches!(
                (&auth, provider),
                (
                    CodexAuth::ApiKey(_) | CodexAuth::Chatgpt(_) | CodexAuth::ChatgptAuthTokens(_),
                    ProviderName::OpenAI,
                ) | (
                    CodexAuth::AnthropicApiKey(_) | CodexAuth::AnthropicOAuth(_),
                    ProviderName::Anthropic,
                )
            );
            if is_match {
                return Some(auth);
            }
        }

        // If no cached match, try loading specifically for the provider
        match provider {
            ProviderName::Anthropic => {
                // Check persistent storage first — OAuth tokens take priority
                // over env var so mid-session provider switches work correctly.
                if let Ok(Some(v2)) =
                    load_auth_dot_json_v2(&self.orbit_code_home, self.auth_credentials_store_mode)
                    && let Some(provider_auth) = v2.provider_auth(ProviderName::Anthropic)
                {
                    return codex_auth_from_provider_auth(provider_auth);
                }
                // Fall back to ANTHROPIC_API_KEY env var
                if let Ok(key) = std::env::var("ANTHROPIC_API_KEY")
                    && !key.is_empty()
                {
                    return Some(CodexAuth::AnthropicApiKey(AnthropicApiKeyAuth::new(key)));
                }
                None
            }
            ProviderName::OpenAI => {
                // Check OPENAI_API_KEY env var
                if let Ok(key) = std::env::var("OPENAI_API_KEY")
                    && !key.is_empty()
                {
                    let client = crate::default_client::create_client();
                    return Some(CodexAuth::from_api_key_with_client(&key, client));
                }
                // Check persistent v2 storage for OpenAI provider
                if let Ok(Some(v2)) =
                    load_auth_dot_json_v2(&self.orbit_code_home, self.auth_credentials_store_mode)
                    && v2.provider_auth(ProviderName::OpenAI).is_some()
                {
                    let auth_dot_json = v2.to_v1_openai();
                    let client = crate::default_client::create_client();
                    if let Ok(auth) = CodexAuth::from_auth_dot_json(
                        &self.orbit_code_home,
                        auth_dot_json,
                        self.auth_credentials_store_mode,
                        client,
                    ) {
                        return Some(auth);
                    }
                }
                None
            }
        }
    }

    /// Current cached auth (clone). May be `None` if not logged in or load failed.
    /// Refreshes cached ChatGPT tokens if they are stale before returning.
    pub async fn auth(&self) -> Option<CodexAuth> {
        let auth = self.auth_cached()?;
        if let Err(err) = self.refresh_if_stale(&auth).await {
            tracing::error!("Failed to refresh token: {}", err);
            return Some(auth);
        }
        self.auth_cached()
    }

    /// Force a reload of the auth information from auth.json. Returns
    /// whether the auth value changed.
    pub fn reload(&self) -> bool {
        tracing::info!("Reloading auth");
        let new_auth = self.load_auth_from_storage();
        self.set_cached_auth(new_auth)
    }

    pub(super) fn reload_if_account_id_matches(
        &self,
        expected_account_id: Option<&str>,
    ) -> ReloadOutcome {
        let expected_account_id = match expected_account_id {
            Some(account_id) => account_id,
            None => {
                tracing::info!("Skipping auth reload because no account id is available.");
                return ReloadOutcome::Skipped;
            }
        };

        let new_auth = self.load_auth_from_storage();
        let new_account_id = new_auth.as_ref().and_then(CodexAuth::get_account_id);

        if new_account_id.as_deref() != Some(expected_account_id) {
            let found_account_id = new_account_id.as_deref().unwrap_or("unknown");
            tracing::info!(
                "Skipping auth reload due to account id mismatch (expected: {expected_account_id}, found: {found_account_id})"
            );
            return ReloadOutcome::Skipped;
        }

        tracing::info!("Reloading auth for account {expected_account_id}");
        let cached_before_reload = self.auth_cached();
        let auth_changed =
            !Self::auths_equal_for_refresh(cached_before_reload.as_ref(), new_auth.as_ref());
        self.set_cached_auth(new_auth);
        if auth_changed {
            ReloadOutcome::ReloadedChanged
        } else {
            ReloadOutcome::ReloadedNoChange
        }
    }

    fn auths_equal_for_refresh(a: Option<&CodexAuth>, b: Option<&CodexAuth>) -> bool {
        match (a, b) {
            (None, None) => true,
            (Some(a), Some(b)) => match (a.api_auth_mode(), b.api_auth_mode()) {
                (ApiAuthMode::ApiKey, ApiAuthMode::ApiKey) => a.api_key() == b.api_key(),
                (ApiAuthMode::Chatgpt, ApiAuthMode::Chatgpt)
                | (ApiAuthMode::ChatgptAuthTokens, ApiAuthMode::ChatgptAuthTokens) => {
                    a.get_current_auth_json() == b.get_current_auth_json()
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub(super) fn auths_equal(a: Option<&CodexAuth>, b: Option<&CodexAuth>) -> bool {
        match (a, b) {
            (None, None) => true,
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }

    fn load_auth_from_storage(&self) -> Option<CodexAuth> {
        load_auth(
            &self.orbit_code_home,
            self.enable_orbit_code_api_key_env,
            self.auth_credentials_store_mode,
        )
        .ok()
        .flatten()
    }

    fn set_cached_auth(&self, new_auth: Option<CodexAuth>) -> bool {
        if let Ok(mut guard) = self.inner.write() {
            let previous = guard.auth.as_ref();
            let changed = !AuthManager::auths_equal(previous, new_auth.as_ref());
            tracing::info!("Reloaded auth, changed: {changed}");
            guard.auth = new_auth;
            changed
        } else {
            false
        }
    }

    pub fn set_external_auth_refresher(&self, refresher: Arc<dyn ExternalAuthRefresher>) {
        if let Ok(mut guard) = self.inner.write() {
            guard.external_refresher = Some(refresher);
        }
    }

    pub fn clear_external_auth_refresher(&self) {
        if let Ok(mut guard) = self.inner.write() {
            guard.external_refresher = None;
        }
    }

    pub fn set_forced_chatgpt_workspace_id(&self, workspace_id: Option<String>) {
        if let Ok(mut guard) = self.forced_chatgpt_workspace_id.write() {
            *guard = workspace_id;
        }
    }

    pub fn forced_chatgpt_workspace_id(&self) -> Option<String> {
        self.forced_chatgpt_workspace_id
            .read()
            .ok()
            .and_then(|guard| guard.clone())
    }

    pub fn has_external_auth_refresher(&self) -> bool {
        self.inner
            .read()
            .ok()
            .map(|guard| guard.external_refresher.is_some())
            .unwrap_or(false)
    }

    pub fn is_external_auth_active(&self) -> bool {
        self.auth_cached()
            .as_ref()
            .is_some_and(CodexAuth::is_external_chatgpt_tokens)
    }

    pub fn orbit_code_api_key_env_enabled(&self) -> bool {
        self.enable_orbit_code_api_key_env
    }

    /// The Orbit Code home directory used to locate auth storage.
    pub(crate) fn orbit_code_home(&self) -> &Path {
        &self.orbit_code_home
    }

    /// The credential storage mode (file, keyring, auto, ephemeral).
    pub(crate) fn auth_credentials_store_mode(&self) -> AuthCredentialsStoreMode {
        self.auth_credentials_store_mode
    }

    /// Convenience constructor returning an `Arc` wrapper.
    pub fn shared(
        orbit_code_home: PathBuf,
        enable_orbit_code_api_key_env: bool,
        auth_credentials_store_mode: AuthCredentialsStoreMode,
    ) -> Arc<Self> {
        Arc::new(Self::new(
            orbit_code_home,
            enable_orbit_code_api_key_env,
            auth_credentials_store_mode,
        ))
    }

    pub fn unauthorized_recovery(self: &Arc<Self>) -> UnauthorizedRecovery {
        UnauthorizedRecovery::new(Arc::clone(self))
    }

    /// Attempt to refresh the token by first performing a guarded reload. Auth
    /// is reloaded from storage only when the account id matches the currently
    /// cached account id. If the persisted token differs from the cached token, we
    /// can assume that some other instance already refreshed it. If the persisted
    /// token is the same as the cached, then ask the token authority to refresh.
    pub async fn refresh_token(&self) -> Result<(), RefreshTokenError> {
        let auth_before_reload = self.auth_cached();
        let expected_account_id = auth_before_reload
            .as_ref()
            .and_then(CodexAuth::get_account_id);

        match self.reload_if_account_id_matches(expected_account_id.as_deref()) {
            ReloadOutcome::ReloadedChanged => {
                tracing::info!("Skipping token refresh because auth changed after guarded reload.");
                Ok(())
            }
            ReloadOutcome::ReloadedNoChange => self.refresh_token_from_authority().await,
            ReloadOutcome::Skipped => {
                Err(RefreshTokenError::Permanent(RefreshTokenFailedError::new(
                    RefreshTokenFailedReason::Other,
                    REFRESH_TOKEN_ACCOUNT_MISMATCH_MESSAGE.to_string(),
                )))
            }
        }
    }

    /// Attempt to refresh the current auth token from the authority that issued
    /// the token. On success, reloads the auth state from disk so other components
    /// observe refreshed token. If the token refresh fails, returns the error to
    /// the caller.
    pub async fn refresh_token_from_authority(&self) -> Result<(), RefreshTokenError> {
        tracing::info!("Refreshing token");

        let auth = match self.auth_cached() {
            Some(auth) => auth,
            None => return Ok(()),
        };
        match auth {
            CodexAuth::ChatgptAuthTokens(_) => {
                self.refresh_external_auth(ExternalAuthRefreshReason::Unauthorized)
                    .await
            }
            CodexAuth::Chatgpt(chatgpt_auth) => {
                let token_data = chatgpt_auth.current_token_data().ok_or_else(|| {
                    RefreshTokenError::Transient(std::io::Error::other(
                        "Token data is not available.",
                    ))
                })?;
                self.refresh_and_persist_chatgpt_token(&chatgpt_auth, token_data.refresh_token)
                    .await?;
                Ok(())
            }
            // API key and Anthropic API key auth don't need token refresh.
            CodexAuth::ApiKey(_) | CodexAuth::AnthropicApiKey(_) => Ok(()),
            // Anthropic OAuth refresh will be handled separately when needed.
            CodexAuth::AnthropicOAuth(_) => Ok(()),
        }
    }

    /// Log out by deleting the on‑disk auth.json (if present). Returns Ok(true)
    /// if a file was removed, Ok(false) if no auth file existed. On success,
    /// reloads the in‑memory auth cache so callers immediately observe the
    /// unauthenticated state.
    pub fn logout(&self) -> std::io::Result<bool> {
        let removed = logout_all_stores(&self.orbit_code_home, self.auth_credentials_store_mode)?;
        // Always reload to clear any cached auth (even if file absent).
        self.reload();
        Ok(removed)
    }

    /// Proactively refresh Anthropic OAuth if the token is expiring soon.
    /// Best-effort: failures are logged but don't prevent the request from proceeding
    /// (the stale token might still work, and 401 recovery handles the rest).
    pub async fn refresh_anthropic_oauth_if_needed(&self) {
        crate::anthropic_auth::refresh_if_needed(self).await;
    }

    /// Force-refresh an Anthropic OAuth token. Unlike proactive refresh, this
    /// always refreshes (no expiry check) and returns a typed error.
    pub async fn force_refresh_anthropic_oauth(
        &self,
    ) -> std::result::Result<(), RefreshTokenError> {
        crate::anthropic_auth::force_refresh(self).await
    }

    pub fn get_api_auth_mode(&self) -> Option<ApiAuthMode> {
        self.auth_cached().as_ref().map(CodexAuth::api_auth_mode)
    }

    pub fn auth_mode(&self) -> Option<AuthMode> {
        self.auth_cached().as_ref().map(CodexAuth::auth_mode)
    }

    pub(crate) async fn refresh_if_stale(
        &self,
        auth: &CodexAuth,
    ) -> Result<bool, RefreshTokenError> {
        let chatgpt_auth = match auth {
            CodexAuth::Chatgpt(chatgpt_auth) => chatgpt_auth,
            _ => return Ok(false),
        };

        let auth_dot_json = match chatgpt_auth.current_auth_json() {
            Some(auth_dot_json) => auth_dot_json,
            None => return Ok(false),
        };
        let tokens = match auth_dot_json.tokens {
            Some(tokens) => tokens,
            None => return Ok(false),
        };
        let last_refresh = match auth_dot_json.last_refresh {
            Some(last_refresh) => last_refresh,
            None => return Ok(false),
        };
        if last_refresh >= Utc::now() - chrono::Duration::days(TOKEN_REFRESH_INTERVAL) {
            return Ok(false);
        }
        self.refresh_and_persist_chatgpt_token(chatgpt_auth, tokens.refresh_token)
            .await?;
        Ok(true)
    }

    pub(super) async fn refresh_external_auth(
        &self,
        reason: ExternalAuthRefreshReason,
    ) -> Result<(), RefreshTokenError> {
        let forced_chatgpt_workspace_id = self.forced_chatgpt_workspace_id();
        let refresher = match self.inner.read() {
            Ok(guard) => guard.external_refresher.clone(),
            Err(_) => {
                return Err(RefreshTokenError::Transient(std::io::Error::other(
                    "failed to read external auth state",
                )));
            }
        };

        let Some(refresher) = refresher else {
            return Err(RefreshTokenError::Transient(std::io::Error::other(
                "external auth refresher is not configured",
            )));
        };

        let previous_account_id = self
            .auth_cached()
            .as_ref()
            .and_then(CodexAuth::get_account_id);
        let context = ExternalAuthRefreshContext {
            reason,
            previous_account_id,
        };

        let refreshed = refresher.refresh(context).await?;
        if let Some(expected_workspace_id) = forced_chatgpt_workspace_id.as_deref()
            && refreshed.chatgpt_account_id != expected_workspace_id
        {
            return Err(RefreshTokenError::Transient(std::io::Error::other(
                format!(
                    "external auth refresh returned workspace {:?}, expected {expected_workspace_id:?}",
                    refreshed.chatgpt_account_id,
                ),
            )));
        }
        let auth_dot_json =
            AuthDotJson::from_external_tokens(&refreshed).map_err(RefreshTokenError::Transient)?;
        save_auth(
            &self.orbit_code_home,
            &auth_dot_json,
            AuthCredentialsStoreMode::Ephemeral,
        )
        .map_err(RefreshTokenError::Transient)?;
        self.reload();
        Ok(())
    }

    // Refreshes ChatGPT OAuth tokens, persists the updated auth state, and
    // reloads the in-memory cache so callers immediately observe new tokens.
    async fn refresh_and_persist_chatgpt_token(
        &self,
        auth: &ChatgptAuth,
        refresh_token: String,
    ) -> Result<(), RefreshTokenError> {
        let refresh_response = request_chatgpt_token_refresh(refresh_token, auth.client()).await?;

        persist_tokens(
            auth.storage(),
            refresh_response.id_token,
            refresh_response.access_token,
            refresh_response.refresh_token,
        )
        .map_err(RefreshTokenError::from)?;
        self.reload();

        Ok(())
    }
}

// ── ChatGPT token refresh primitives ────────────────────────────────

#[derive(Serialize)]
struct RefreshRequest {
    client_id: &'static str,
    grant_type: &'static str,
    refresh_token: String,
}

#[derive(Deserialize, Clone)]
struct RefreshResponse {
    id_token: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
}

fn refresh_token_endpoint() -> String {
    std::env::var(REFRESH_TOKEN_URL_OVERRIDE_ENV_VAR)
        .unwrap_or_else(|_| REFRESH_TOKEN_URL.to_string())
}

// Requests refreshed ChatGPT OAuth tokens from the auth service using a refresh token.
// The caller is responsible for persisting any returned tokens.
async fn request_chatgpt_token_refresh(
    refresh_token: String,
    client: &CodexHttpClient,
) -> Result<RefreshResponse, RefreshTokenError> {
    let refresh_request = RefreshRequest {
        client_id: CLIENT_ID,
        grant_type: "refresh_token",
        refresh_token,
    };

    let endpoint = refresh_token_endpoint();

    // Use shared client factory to include standard headers
    let response = client
        .post(endpoint.as_str())
        .header("Content-Type", "application/json")
        .json(&refresh_request)
        .send()
        .await
        .map_err(|err| RefreshTokenError::Transient(std::io::Error::other(err)))?;

    let status = response.status();
    if status.is_success() {
        let refresh_response = response
            .json::<RefreshResponse>()
            .await
            .map_err(|err| RefreshTokenError::Transient(std::io::Error::other(err)))?;
        Ok(refresh_response)
    } else {
        let body = response.text().await.unwrap_or_default();
        tracing::error!("Failed to refresh token: {status}: {body}");
        if status == StatusCode::UNAUTHORIZED {
            let failed = classify_refresh_token_failure(&body);
            Err(RefreshTokenError::Permanent(failed))
        } else {
            let message = try_parse_error_message(&body);
            Err(RefreshTokenError::Transient(std::io::Error::other(
                format!("Failed to refresh token: {status}: {message}"),
            )))
        }
    }
}

fn classify_refresh_token_failure(body: &str) -> RefreshTokenFailedError {
    let code = extract_refresh_token_error_code(body);

    let normalized_code = code.as_deref().map(str::to_ascii_lowercase);
    let reason = match normalized_code.as_deref() {
        Some("refresh_token_expired") => RefreshTokenFailedReason::Expired,
        Some("refresh_token_reused") => RefreshTokenFailedReason::Exhausted,
        Some("refresh_token_invalidated") => RefreshTokenFailedReason::Revoked,
        _ => RefreshTokenFailedReason::Other,
    };

    if reason == RefreshTokenFailedReason::Other {
        tracing::warn!(
            backend_code = normalized_code.as_deref(),
            backend_body = body,
            "Encountered unknown 401 response while refreshing token"
        );
    }

    let message = match reason {
        RefreshTokenFailedReason::Expired => REFRESH_TOKEN_EXPIRED_MESSAGE.to_string(),
        RefreshTokenFailedReason::Exhausted => REFRESH_TOKEN_REUSED_MESSAGE.to_string(),
        RefreshTokenFailedReason::Revoked => REFRESH_TOKEN_INVALIDATED_MESSAGE.to_string(),
        RefreshTokenFailedReason::Other => REFRESH_TOKEN_UNKNOWN_MESSAGE.to_string(),
    };

    RefreshTokenFailedError::new(reason, message)
}

fn extract_refresh_token_error_code(body: &str) -> Option<String> {
    if body.trim().is_empty() {
        return None;
    }

    let serde_json::Value::Object(map) = serde_json::from_str::<serde_json::Value>(body).ok()?
    else {
        return None;
    };

    if let Some(error_value) = map.get("error") {
        match error_value {
            serde_json::Value::Object(obj) => {
                if let Some(code) = obj.get("code").and_then(serde_json::Value::as_str) {
                    return Some(code.to_string());
                }
            }
            serde_json::Value::String(code) => {
                return Some(code.to_string());
            }
            _ => {}
        }
    }

    map.get("code")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
}
