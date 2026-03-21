mod storage;

use async_trait::async_trait;
use chrono::Utc;
use reqwest::StatusCode;
use serde::Deserialize;
use serde::Serialize;
#[cfg(test)]
use serial_test::serial;
use std::env;
use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

use orbit_code_app_server_protocol::AuthMode as ApiAuthMode;
use orbit_code_otel::TelemetryAuthMode;
use orbit_code_protocol::config_types::ForcedLoginMethod;

pub use crate::anthropic_auth::AnthropicApiKeyAuth;
pub use crate::anthropic_auth::AnthropicOAuthAuth;
pub use crate::auth::storage::AuthCredentialsStoreMode;
pub use crate::auth::storage::AuthDotJson;
pub use crate::auth::storage::AuthDotJsonV2;
pub(crate) use crate::auth::storage::AuthStorageBackend;
pub use crate::auth::storage::ProviderAuth;
pub use crate::auth::storage::ProviderName;
pub(crate) use crate::auth::storage::create_auth_storage;
use crate::config::Config;
use crate::error::RefreshTokenFailedError;
use crate::error::RefreshTokenFailedReason;
use crate::token_data::KnownPlan as InternalKnownPlan;
use crate::token_data::PlanType as InternalPlanType;
use crate::token_data::TokenData;
use crate::token_data::parse_chatgpt_jwt_claims;
use crate::util::try_parse_error_message;
use orbit_code_client::CodexHttpClient;
use orbit_code_protocol::account::PlanType as AccountPlanType;
use serde_json::Value;
use thiserror::Error;

/// Account type for the current user.
///
/// This is used internally to determine the base URL for generating responses,
/// and to gate ChatGPT-only behaviors like rate limits and available models (as
/// opposed to API key-based auth).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthMode {
    ApiKey,
    Chatgpt,
    AnthropicApiKey,
    AnthropicOAuth,
}

impl From<AuthMode> for TelemetryAuthMode {
    fn from(mode: AuthMode) -> Self {
        match mode {
            AuthMode::ApiKey => TelemetryAuthMode::ApiKey,
            AuthMode::Chatgpt => TelemetryAuthMode::Chatgpt,
            AuthMode::AnthropicApiKey => TelemetryAuthMode::AnthropicApiKey,
            AuthMode::AnthropicOAuth => TelemetryAuthMode::AnthropicOAuth,
        }
    }
}

/// Authentication mechanism used by the current user.
#[derive(Debug, Clone)]
pub enum CodexAuth {
    ApiKey(ApiKeyAuth),
    Chatgpt(ChatgptAuth),
    ChatgptAuthTokens(ChatgptAuthTokens),
    AnthropicApiKey(AnthropicApiKeyAuth),
    AnthropicOAuth(AnthropicOAuthAuth),
}

#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    api_key: String,
}

#[derive(Debug, Clone)]
pub struct ChatgptAuth {
    state: ChatgptAuthState,
    storage: Arc<dyn AuthStorageBackend>,
}

#[derive(Debug, Clone)]
pub struct ChatgptAuthTokens {
    state: ChatgptAuthState,
}

#[derive(Debug, Clone)]
struct ChatgptAuthState {
    auth_dot_json: Arc<Mutex<Option<AuthDotJson>>>,
    client: CodexHttpClient,
}

impl PartialEq for CodexAuth {
    fn eq(&self, other: &Self) -> bool {
        self.api_auth_mode() == other.api_auth_mode()
    }
}

// TODO(pakrym): use token exp field to check for expiration instead
const TOKEN_REFRESH_INTERVAL: i64 = 8;

const REFRESH_TOKEN_EXPIRED_MESSAGE: &str = "Your access token could not be refreshed because your refresh token has expired. Please log out and sign in again.";
const REFRESH_TOKEN_REUSED_MESSAGE: &str = "Your access token could not be refreshed because your refresh token was already used. Please log out and sign in again.";
const REFRESH_TOKEN_INVALIDATED_MESSAGE: &str = "Your access token could not be refreshed because your refresh token was revoked. Please log out and sign in again.";
const REFRESH_TOKEN_UNKNOWN_MESSAGE: &str =
    "Your access token could not be refreshed. Please log out and sign in again.";
const REFRESH_TOKEN_ACCOUNT_MISMATCH_MESSAGE: &str = "Your access token could not be refreshed because you have since logged out or signed in to another account. Please sign in again.";
const REFRESH_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
pub const REFRESH_TOKEN_URL_OVERRIDE_ENV_VAR: &str = "ORBIT_REFRESH_TOKEN_URL_OVERRIDE";

#[derive(Debug, Error)]
pub enum RefreshTokenError {
    #[error("{0}")]
    Permanent(#[from] RefreshTokenFailedError),
    #[error(transparent)]
    Transient(#[from] std::io::Error),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalAuthTokens {
    pub access_token: String,
    pub chatgpt_account_id: String,
    pub chatgpt_plan_type: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExternalAuthRefreshReason {
    Unauthorized,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalAuthRefreshContext {
    pub reason: ExternalAuthRefreshReason,
    pub previous_account_id: Option<String>,
}

#[async_trait]
pub trait ExternalAuthRefresher: Send + Sync {
    async fn refresh(
        &self,
        context: ExternalAuthRefreshContext,
    ) -> std::io::Result<ExternalAuthTokens>;
}

impl RefreshTokenError {
    pub fn failed_reason(&self) -> Option<RefreshTokenFailedReason> {
        match self {
            Self::Permanent(error) => Some(error.reason),
            Self::Transient(_) => None,
        }
    }
}

impl From<RefreshTokenError> for std::io::Error {
    fn from(err: RefreshTokenError) -> Self {
        match err {
            RefreshTokenError::Permanent(failed) => std::io::Error::other(failed),
            RefreshTokenError::Transient(inner) => inner,
        }
    }
}

impl CodexAuth {
    fn from_auth_dot_json(
        orbit_code_home: &Path,
        auth_dot_json: AuthDotJson,
        auth_credentials_store_mode: AuthCredentialsStoreMode,
        client: CodexHttpClient,
    ) -> std::io::Result<Self> {
        let auth_mode = auth_dot_json.resolved_mode();
        if auth_mode == ApiAuthMode::ApiKey {
            let Some(api_key) = auth_dot_json.openai_api_key.as_deref() else {
                return Err(std::io::Error::other("API key auth is missing a key."));
            };
            return Ok(CodexAuth::from_api_key_with_client(api_key, client));
        }

        let storage_mode = auth_dot_json.storage_mode(auth_credentials_store_mode);
        let state = ChatgptAuthState {
            auth_dot_json: Arc::new(Mutex::new(Some(auth_dot_json))),
            client,
        };

        match auth_mode {
            ApiAuthMode::Chatgpt => {
                let storage = create_auth_storage(orbit_code_home.to_path_buf(), storage_mode);
                Ok(Self::Chatgpt(ChatgptAuth { state, storage }))
            }
            ApiAuthMode::ChatgptAuthTokens => {
                Ok(Self::ChatgptAuthTokens(ChatgptAuthTokens { state }))
            }
            ApiAuthMode::ApiKey => unreachable!("api key mode is handled above"),
            ApiAuthMode::AnthropicApiKey | ApiAuthMode::AnthropicOAuth => {
                unreachable!("anthropic auth modes are handled by the Anthropic provider")
            }
        }
    }

    /// Loads the available auth information from auth storage.
    pub fn from_auth_storage(
        orbit_code_home: &Path,
        auth_credentials_store_mode: AuthCredentialsStoreMode,
    ) -> std::io::Result<Option<Self>> {
        load_auth(
            orbit_code_home,
            /*enable_orbit_code_api_key_env*/ false,
            auth_credentials_store_mode,
        )
    }

    pub fn auth_mode(&self) -> AuthMode {
        match self {
            Self::ApiKey(_) => AuthMode::ApiKey,
            Self::Chatgpt(_) | Self::ChatgptAuthTokens(_) => AuthMode::Chatgpt,
            Self::AnthropicApiKey(_) => AuthMode::AnthropicApiKey,
            Self::AnthropicOAuth(_) => AuthMode::AnthropicOAuth,
        }
    }

    pub fn api_auth_mode(&self) -> ApiAuthMode {
        match self {
            Self::ApiKey(_) => ApiAuthMode::ApiKey,
            Self::Chatgpt(_) => ApiAuthMode::Chatgpt,
            Self::ChatgptAuthTokens(_) => ApiAuthMode::ChatgptAuthTokens,
            Self::AnthropicApiKey(_) => ApiAuthMode::AnthropicApiKey,
            Self::AnthropicOAuth(_) => ApiAuthMode::AnthropicOAuth,
        }
    }

    pub fn is_api_key_auth(&self) -> bool {
        self.auth_mode() == AuthMode::ApiKey
    }

    pub fn is_chatgpt_auth(&self) -> bool {
        self.auth_mode() == AuthMode::Chatgpt
    }

    pub fn is_external_chatgpt_tokens(&self) -> bool {
        matches!(self, Self::ChatgptAuthTokens(_))
    }

    /// Returns `None` if not an API key auth mode.
    pub fn api_key(&self) -> Option<&str> {
        match self {
            Self::ApiKey(auth) => Some(auth.api_key.as_str()),
            Self::AnthropicApiKey(auth) => Some(auth.api_key()),
            Self::Chatgpt(_) | Self::ChatgptAuthTokens(_) | Self::AnthropicOAuth(_) => None,
        }
    }

    /// Returns `Err` if `is_chatgpt_auth()` is false.
    pub fn get_token_data(&self) -> Result<TokenData, std::io::Error> {
        let auth_dot_json: Option<AuthDotJson> = self.get_current_auth_json();
        match auth_dot_json {
            Some(AuthDotJson {
                tokens: Some(tokens),
                last_refresh: Some(_),
                ..
            }) => Ok(tokens),
            _ => Err(std::io::Error::other("Token data is not available.")),
        }
    }

    /// Returns the token string used for bearer authentication.
    pub fn get_token(&self) -> Result<String, std::io::Error> {
        match self {
            Self::ApiKey(auth) => Ok(auth.api_key.clone()),
            Self::AnthropicApiKey(auth) => Ok(auth.api_key().to_string()),
            Self::AnthropicOAuth(auth) => Ok(auth.access_token().to_string()),
            Self::Chatgpt(_) | Self::ChatgptAuthTokens(_) => {
                let access_token = self.get_token_data()?.access_token;
                Ok(access_token)
            }
        }
    }

    /// Returns `None` if `is_chatgpt_auth()` is false.
    pub fn get_account_id(&self) -> Option<String> {
        self.get_current_token_data().and_then(|t| t.account_id)
    }

    /// Returns `None` if `is_chatgpt_auth()` is false.
    pub fn get_account_email(&self) -> Option<String> {
        self.get_current_token_data().and_then(|t| t.id_token.email)
    }

    /// Returns `None` if `is_chatgpt_auth()` is false.
    pub fn get_chatgpt_user_id(&self) -> Option<String> {
        self.get_current_token_data()
            .and_then(|t| t.id_token.chatgpt_user_id)
    }

    /// Account-facing plan classification derived from the current token.
    /// Returns a high-level `AccountPlanType` (e.g., Free/Plus/Pro/Team/…)
    /// mapped from the ID token's internal plan value. Prefer this when you
    /// need to make UI or product decisions based on the user's subscription.
    /// When ChatGPT auth is active but the token omits the plan claim, report
    /// `Unknown` instead of treating the account as invalid.
    pub fn account_plan_type(&self) -> Option<AccountPlanType> {
        let map_known = |kp: &InternalKnownPlan| match kp {
            InternalKnownPlan::Free => AccountPlanType::Free,
            InternalKnownPlan::Go => AccountPlanType::Go,
            InternalKnownPlan::Plus => AccountPlanType::Plus,
            InternalKnownPlan::Pro => AccountPlanType::Pro,
            InternalKnownPlan::Team => AccountPlanType::Team,
            InternalKnownPlan::Business => AccountPlanType::Business,
            InternalKnownPlan::Enterprise => AccountPlanType::Enterprise,
            InternalKnownPlan::Edu => AccountPlanType::Edu,
        };

        self.get_current_token_data().map(|t| {
            t.id_token
                .chatgpt_plan_type
                .map(|pt| match pt {
                    InternalPlanType::Known(k) => map_known(&k),
                    InternalPlanType::Unknown(_) => AccountPlanType::Unknown,
                })
                .unwrap_or(AccountPlanType::Unknown)
        })
    }

    /// Returns `None` if `is_chatgpt_auth()` is false.
    fn get_current_auth_json(&self) -> Option<AuthDotJson> {
        let state = match self {
            Self::Chatgpt(auth) => &auth.state,
            Self::ChatgptAuthTokens(auth) => &auth.state,
            Self::ApiKey(_) | Self::AnthropicApiKey(_) | Self::AnthropicOAuth(_) => return None,
        };
        #[expect(clippy::unwrap_used)]
        state.auth_dot_json.lock().unwrap().clone()
    }

    /// Returns `None` if `is_chatgpt_auth()` is false.
    fn get_current_token_data(&self) -> Option<TokenData> {
        self.get_current_auth_json().and_then(|t| t.tokens)
    }

    /// Consider this private to integration tests.
    pub fn create_dummy_chatgpt_auth_for_testing() -> Self {
        let auth_dot_json = AuthDotJson {
            auth_mode: Some(ApiAuthMode::Chatgpt),
            openai_api_key: None,
            tokens: Some(TokenData {
                id_token: Default::default(),
                access_token: "Access Token".to_string(),
                refresh_token: "test".to_string(),
                account_id: Some("account_id".to_string()),
            }),
            last_refresh: Some(Utc::now()),
        };

        let client = crate::default_client::create_client();
        let state = ChatgptAuthState {
            auth_dot_json: Arc::new(Mutex::new(Some(auth_dot_json))),
            client,
        };
        let storage = create_auth_storage(PathBuf::new(), AuthCredentialsStoreMode::File);
        Self::Chatgpt(ChatgptAuth { state, storage })
    }

    fn from_api_key_with_client(api_key: &str, _client: CodexHttpClient) -> Self {
        Self::ApiKey(ApiKeyAuth {
            api_key: api_key.to_owned(),
        })
    }

    pub fn from_api_key(api_key: &str) -> Self {
        Self::from_api_key_with_client(api_key, crate::default_client::create_client())
    }
}

impl ChatgptAuth {
    fn current_auth_json(&self) -> Option<AuthDotJson> {
        #[expect(clippy::unwrap_used)]
        self.state.auth_dot_json.lock().unwrap().clone()
    }

    fn current_token_data(&self) -> Option<TokenData> {
        self.current_auth_json().and_then(|auth| auth.tokens)
    }

    fn storage(&self) -> &Arc<dyn AuthStorageBackend> {
        &self.storage
    }

    fn client(&self) -> &CodexHttpClient {
        &self.state.client
    }
}

pub const OPENAI_API_KEY_ENV_VAR: &str = "OPENAI_API_KEY";
pub const ORBIT_API_KEY_ENV_VAR: &str = "ORBIT_API_KEY";
pub const LEGACY_CODEX_API_KEY_ENV_VAR: &str = "CODEX_API_KEY";

pub fn read_openai_api_key_from_env() -> Option<String> {
    env::var(OPENAI_API_KEY_ENV_VAR)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn read_orbit_api_key_from_env() -> Option<String> {
    env::var(ORBIT_API_KEY_ENV_VAR)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            env::var(LEGACY_CODEX_API_KEY_ENV_VAR)
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
}

pub fn read_orbit_code_api_key_from_env() -> Option<String> {
    read_orbit_api_key_from_env()
}

/// Delete the auth.json file inside `orbit_code_home` if it exists. Returns `Ok(true)`
/// if a file was removed, `Ok(false)` if no auth file was present.
pub fn logout(
    orbit_code_home: &Path,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
) -> std::io::Result<bool> {
    let storage = create_auth_storage(orbit_code_home.to_path_buf(), auth_credentials_store_mode);
    storage.delete()
}

/// Delete auth credentials for a single provider, preserving credentials for
/// other providers. Returns `Ok(true)` if the provider had stored credentials
/// that were removed, `Ok(false)` if the provider had no credentials.
pub fn logout_provider(
    orbit_code_home: &Path,
    provider: ProviderName,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
) -> std::io::Result<bool> {
    let storage = create_auth_storage(orbit_code_home.to_path_buf(), auth_credentials_store_mode);
    storage.delete_provider(provider)
}

/// Writes an `auth.json` that contains only the API key.
pub fn login_with_api_key(
    orbit_code_home: &Path,
    api_key: &str,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
) -> std::io::Result<()> {
    let auth_dot_json = AuthDotJson {
        auth_mode: Some(ApiAuthMode::ApiKey),
        openai_api_key: Some(api_key.to_string()),
        tokens: None,
        last_refresh: None,
    };
    save_auth(orbit_code_home, &auth_dot_json, auth_credentials_store_mode)
}

/// Writes an in-memory auth payload for externally managed ChatGPT tokens.
pub fn login_with_chatgpt_auth_tokens(
    orbit_code_home: &Path,
    access_token: &str,
    chatgpt_account_id: &str,
    chatgpt_plan_type: Option<&str>,
) -> std::io::Result<()> {
    let auth_dot_json = AuthDotJson::from_external_access_token(
        access_token,
        chatgpt_account_id,
        chatgpt_plan_type,
    )?;
    save_auth(
        orbit_code_home,
        &auth_dot_json,
        AuthCredentialsStoreMode::Ephemeral,
    )
}

/// Persist the provided auth payload using the specified backend.
/// Converts v1 AuthDotJson to v2 format and merges into existing storage,
/// preserving credentials for other providers (e.g., saving OpenAI auth
/// does not clobber an existing Anthropic entry).
pub fn save_auth(
    orbit_code_home: &Path,
    auth: &AuthDotJson,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
) -> std::io::Result<()> {
    let storage = create_auth_storage(orbit_code_home.to_path_buf(), auth_credentials_store_mode);
    let new_v2 = AuthDotJsonV2::from(auth.clone());
    // Merge into existing v2 storage to preserve other providers.
    let merged = match storage.load()? {
        Some(mut existing) => {
            for (provider, provider_auth) in &new_v2.providers {
                existing.set_provider_auth(*provider, provider_auth.clone());
            }
            existing
        }
        None => new_v2,
    };
    storage.save(&merged)
}

/// Persist a v2 auth payload using the specified backend.
/// Merges into existing storage to preserve credentials for other providers.
pub fn save_auth_v2(
    orbit_code_home: &Path,
    auth: &AuthDotJsonV2,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
) -> std::io::Result<()> {
    let storage = create_auth_storage(orbit_code_home.to_path_buf(), auth_credentials_store_mode);
    let merged = match storage.load()? {
        Some(mut existing) => {
            for (provider, provider_auth) in &auth.providers {
                existing.set_provider_auth(*provider, provider_auth.clone());
            }
            existing
        }
        None => auth.clone(),
    };
    storage.save(&merged)
}

/// Load CLI auth data using the configured credential store backend.
/// Returns `None` when no credentials are stored. This function is
/// provided only for tests. Production code should not directly load
/// from the auth.json storage. It should use the AuthManager abstraction
/// instead.
///
/// Returns the v1 (OpenAI-centric) view of stored credentials for backward
/// compatibility. Use `load_auth_dot_json_v2` for full multi-provider access.
pub fn load_auth_dot_json(
    orbit_code_home: &Path,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
) -> std::io::Result<Option<AuthDotJson>> {
    let storage = create_auth_storage(orbit_code_home.to_path_buf(), auth_credentials_store_mode);
    match storage.load()? {
        Some(v2) => Ok(Some(v2.to_v1_openai())),
        None => Ok(None),
    }
}

/// Load CLI auth data in v2 format (multi-provider).
pub fn load_auth_dot_json_v2(
    orbit_code_home: &Path,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
) -> std::io::Result<Option<AuthDotJsonV2>> {
    let storage = create_auth_storage(orbit_code_home.to_path_buf(), auth_credentials_store_mode);
    storage.load()
}

pub fn enforce_login_restrictions(config: &Config) -> std::io::Result<()> {
    let Some(auth) = load_auth(
        &config.orbit_code_home,
        /*enable_orbit_code_api_key_env*/ true,
        config.cli_auth_credentials_store_mode,
    )?
    else {
        return Ok(());
    };

    if let Some(required_method) = config.forced_login_method {
        let method_violation = match (required_method, auth.auth_mode()) {
            (ForcedLoginMethod::Api, AuthMode::ApiKey) => None,
            (ForcedLoginMethod::Chatgpt, AuthMode::Chatgpt) => None,
            // Anthropic auth is not subject to OpenAI org enforcement
            (_, AuthMode::AnthropicApiKey | AuthMode::AnthropicOAuth) => None,
            (ForcedLoginMethod::Api, AuthMode::Chatgpt) => Some(
                "API key login is required, but ChatGPT is currently being used. Logging out."
                    .to_string(),
            ),
            (ForcedLoginMethod::Chatgpt, AuthMode::ApiKey) => Some(
                "ChatGPT login is required, but an API key is currently being used. Logging out."
                    .to_string(),
            ),
        };

        if let Some(message) = method_violation {
            return logout_with_message(
                &config.orbit_code_home,
                message,
                config.cli_auth_credentials_store_mode,
            );
        }
    }

    if let Some(expected_account_id) = config.forced_chatgpt_workspace_id.as_deref() {
        if !auth.is_chatgpt_auth() {
            return Ok(());
        }

        let token_data = match auth.get_token_data() {
            Ok(data) => data,
            Err(err) => {
                return logout_with_message(
                    &config.orbit_code_home,
                    format!(
                        "Failed to load ChatGPT credentials while enforcing workspace restrictions: {err}. Logging out."
                    ),
                    config.cli_auth_credentials_store_mode,
                );
            }
        };

        // workspace is the external identifier for account id.
        let chatgpt_account_id = token_data.id_token.chatgpt_account_id.as_deref();
        if chatgpt_account_id != Some(expected_account_id) {
            let message = match chatgpt_account_id {
                Some(actual) => format!(
                    "Login is restricted to workspace {expected_account_id}, but current credentials belong to {actual}. Logging out."
                ),
                None => format!(
                    "Login is restricted to workspace {expected_account_id}, but current credentials lack a workspace identifier. Logging out."
                ),
            };
            return logout_with_message(
                &config.orbit_code_home,
                message,
                config.cli_auth_credentials_store_mode,
            );
        }
    }

    Ok(())
}

fn logout_with_message(
    orbit_code_home: &Path,
    message: String,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
) -> std::io::Result<()> {
    // External auth tokens live in the ephemeral store, but persistent auth may still exist
    // from earlier logins. Clear both so a forced logout truly removes all active auth.
    let removal_result = logout_all_stores(orbit_code_home, auth_credentials_store_mode);
    let error_message = match removal_result {
        Ok(_) => message,
        Err(err) => format!("{message}. Failed to remove auth.json: {err}"),
    };
    Err(std::io::Error::other(error_message))
}

fn logout_all_stores(
    orbit_code_home: &Path,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
) -> std::io::Result<bool> {
    if auth_credentials_store_mode == AuthCredentialsStoreMode::Ephemeral {
        return logout(orbit_code_home, AuthCredentialsStoreMode::Ephemeral);
    }
    let removed_ephemeral = logout(orbit_code_home, AuthCredentialsStoreMode::Ephemeral)?;
    let removed_managed = logout(orbit_code_home, auth_credentials_store_mode)?;
    Ok(removed_ephemeral || removed_managed)
}

fn load_auth(
    orbit_code_home: &Path,
    enable_orbit_code_api_key_env: bool,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
) -> std::io::Result<Option<CodexAuth>> {
    tracing::info!("load_auth: home={}", orbit_code_home.display());
    let build_openai_auth = |v2: &AuthDotJsonV2, storage_mode| {
        let auth_dot_json = v2.to_v1_openai();
        let client = crate::default_client::create_client();
        CodexAuth::from_auth_dot_json(orbit_code_home, auth_dot_json, storage_mode, client)
    };

    // API key via env var takes precedence over any other auth method.
    if enable_orbit_code_api_key_env && let Some(api_key) = read_orbit_api_key_from_env() {
        let client = crate::default_client::create_client();
        return Ok(Some(CodexAuth::from_api_key_with_client(
            api_key.as_str(),
            client,
        )));
    }

    // External ChatGPT auth tokens live in the in-memory (ephemeral) store. Always check this
    // first so external auth takes precedence over any persisted credentials.
    let ephemeral_storage = create_auth_storage(
        orbit_code_home.to_path_buf(),
        AuthCredentialsStoreMode::Ephemeral,
    );
    if let Some(v2) = ephemeral_storage.load()? {
        let auth = build_openai_auth(&v2, AuthCredentialsStoreMode::Ephemeral)?;
        return Ok(Some(auth));
    }

    // If the caller explicitly requested ephemeral auth, there is no persisted fallback.
    if auth_credentials_store_mode == AuthCredentialsStoreMode::Ephemeral {
        return Ok(None);
    }

    // Fall back to the configured persistent store (file/keyring/auto) for managed auth.
    let storage = create_auth_storage(orbit_code_home.to_path_buf(), auth_credentials_store_mode);
    let v2 = match storage.load()? {
        Some(auth) => auth,
        None => return Ok(None),
    };

    // Try OpenAI auth first — but only if the v2 data actually has OpenAI
    // provider data. Without this guard, `to_v1_openai()` returns a default
    // AuthDotJson that creates an empty ChatGPT auth with no token data,
    // causing "Token data is not available" errors downstream.
    let has_openai_data = v2.provider_auth(ProviderName::OpenAI).is_some();
    if has_openai_data && let Ok(auth) = build_openai_auth(&v2, auth_credentials_store_mode) {
        tracing::info!("load_auth: loaded OpenAI auth");
        return Ok(Some(auth));
    }

    // Check for Anthropic provider auth in the v2 data.
    if let Some(provider_auth) = v2.provider_auth(ProviderName::Anthropic)
        && let Some(auth) = codex_auth_from_provider_auth(provider_auth)
    {
        tracing::info!("load_auth: loaded Anthropic auth from v2 storage");
        return Ok(Some(auth));
    }

    tracing::info!("load_auth: no auth found in v2 storage");
    Ok(None)
}

// Persist refreshed tokens into auth storage and update last_refresh.
fn persist_tokens(
    storage: &Arc<dyn AuthStorageBackend>,
    id_token: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
) -> std::io::Result<AuthDotJson> {
    let mut v2 = storage
        .load()?
        .ok_or(std::io::Error::other("Token data is not available."))?;

    // Extract the OpenAI provider auth, modify tokens, and save back.
    let mut auth_dot_json = v2.to_v1_openai();
    let tokens = auth_dot_json.tokens.get_or_insert_with(TokenData::default);
    if let Some(id_token) = id_token {
        tokens.id_token = parse_chatgpt_jwt_claims(&id_token).map_err(std::io::Error::other)?;
    }
    if let Some(access_token) = access_token {
        tokens.access_token = access_token;
    }
    if let Some(refresh_token) = refresh_token {
        tokens.refresh_token = refresh_token;
    }
    auth_dot_json.last_refresh = Some(Utc::now());

    // Convert back to v2 and merge into existing v2 storage (preserving other providers)
    let updated_v1_as_v2 = AuthDotJsonV2::from(auth_dot_json.clone());
    if let Some(openai_auth) = updated_v1_as_v2.provider_auth(ProviderName::OpenAI) {
        v2.set_provider_auth(ProviderName::OpenAI, openai_auth.clone());
    }
    storage.save(&v2)?;
    Ok(auth_dot_json)
}

/// Convert a ProviderAuth entry from storage to a CodexAuth instance.
fn codex_auth_from_provider_auth(provider_auth: &ProviderAuth) -> Option<CodexAuth> {
    match provider_auth {
        ProviderAuth::AnthropicApiKey { key } => Some(CodexAuth::AnthropicApiKey(
            AnthropicApiKeyAuth::new(key.clone()),
        )),
        ProviderAuth::AnthropicOAuth {
            access_token,
            refresh_token,
            expires_at,
        } => Some(CodexAuth::AnthropicOAuth(
            crate::anthropic_auth::AnthropicOAuthAuth {
                access_token: access_token.clone(),
                refresh_token: refresh_token.clone(),
                expires_at: *expires_at,
            },
        )),
        // OpenAI variants are handled by the existing load_auth() path
        ProviderAuth::OpenAiApiKey { .. }
        | ProviderAuth::Chatgpt { .. }
        | ProviderAuth::ChatgptAuthTokens { .. } => None,
    }
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

    let Value::Object(map) = serde_json::from_str::<Value>(body).ok()? else {
        return None;
    };

    if let Some(error_value) = map.get("error") {
        match error_value {
            Value::Object(obj) => {
                if let Some(code) = obj.get("code").and_then(Value::as_str) {
                    return Some(code.to_string());
                }
            }
            Value::String(code) => {
                return Some(code.to_string());
            }
            _ => {}
        }
    }

    map.get("code").and_then(Value::as_str).map(str::to_string)
}

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

// Shared constant for token refresh (client id used for oauth token refresh flow)
pub const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";

fn refresh_token_endpoint() -> String {
    std::env::var(REFRESH_TOKEN_URL_OVERRIDE_ENV_VAR)
        .unwrap_or_else(|_| REFRESH_TOKEN_URL.to_string())
}

impl AuthDotJson {
    fn from_external_tokens(external: &ExternalAuthTokens) -> std::io::Result<Self> {
        let mut token_info =
            parse_chatgpt_jwt_claims(&external.access_token).map_err(std::io::Error::other)?;
        token_info.chatgpt_account_id = Some(external.chatgpt_account_id.clone());
        token_info.chatgpt_plan_type = external
            .chatgpt_plan_type
            .as_deref()
            .map(InternalPlanType::from_raw_value)
            .or(token_info.chatgpt_plan_type)
            .or(Some(InternalPlanType::Unknown("unknown".to_string())));
        let tokens = TokenData {
            id_token: token_info,
            access_token: external.access_token.clone(),
            refresh_token: String::new(),
            account_id: Some(external.chatgpt_account_id.clone()),
        };

        Ok(Self {
            auth_mode: Some(ApiAuthMode::ChatgptAuthTokens),
            openai_api_key: None,
            tokens: Some(tokens),
            last_refresh: Some(Utc::now()),
        })
    }

    fn from_external_access_token(
        access_token: &str,
        chatgpt_account_id: &str,
        chatgpt_plan_type: Option<&str>,
    ) -> std::io::Result<Self> {
        let external = ExternalAuthTokens {
            access_token: access_token.to_string(),
            chatgpt_account_id: chatgpt_account_id.to_string(),
            chatgpt_plan_type: chatgpt_plan_type.map(str::to_string),
        };
        Self::from_external_tokens(&external)
    }

    fn resolved_mode(&self) -> ApiAuthMode {
        if let Some(mode) = self.auth_mode {
            return mode;
        }
        if self.openai_api_key.is_some() {
            return ApiAuthMode::ApiKey;
        }
        ApiAuthMode::Chatgpt
    }

    fn storage_mode(
        &self,
        auth_credentials_store_mode: AuthCredentialsStoreMode,
    ) -> AuthCredentialsStoreMode {
        if self.resolved_mode() == ApiAuthMode::ChatgptAuthTokens {
            AuthCredentialsStoreMode::Ephemeral
        } else {
            auth_credentials_store_mode
        }
    }
}

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

enum UnauthorizedRecoveryStep {
    Reload,
    RefreshToken,
    ExternalRefresh,
    Done,
}

enum ReloadOutcome {
    /// Reload was performed and the cached auth changed
    ReloadedChanged,
    /// Reload was performed and the cached auth remained the same
    ReloadedNoChange,
    /// Reload was skipped (missing or mismatched account id)
    Skipped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnauthorizedRecoveryMode {
    Managed,
    External,
    AnthropicOAuth,
}

// UnauthorizedRecovery is a state machine that handles an attempt to refresh the authentication when requests
// to API fail with 401 status code.
// The client calls next() every time it encounters a 401 error, one time per retry.
// For API key based authentication, we don't do anything and let the error bubble to the user.
//
// For ChatGPT based authentication, we:
// 1. Attempt to reload the auth data from disk. We only reload if the account id matches the one the current process is running as.
// 2. Attempt to refresh the token using OAuth token refresh flow.
// If after both steps the server still responds with 401 we let the error bubble to the user.
//
// For external ChatGPT auth tokens (chatgptAuthTokens), UnauthorizedRecovery does not touch disk or refresh
// tokens locally. Instead it calls the ExternalAuthRefresher (account/chatgptAuthTokens/refresh) to ask the
// parent app for new tokens, stores them in the ephemeral auth store, and retries once.
pub struct UnauthorizedRecovery {
    manager: Arc<AuthManager>,
    step: UnauthorizedRecoveryStep,
    expected_account_id: Option<String>,
    mode: UnauthorizedRecoveryMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnauthorizedRecoveryStepResult {
    auth_state_changed: Option<bool>,
}

impl UnauthorizedRecoveryStepResult {
    pub fn auth_state_changed(&self) -> Option<bool> {
        self.auth_state_changed
    }
}

impl UnauthorizedRecovery {
    fn new(manager: Arc<AuthManager>) -> Self {
        let cached_auth = manager.auth_cached();
        let expected_account_id = cached_auth.as_ref().and_then(CodexAuth::get_account_id);
        let mode = if cached_auth
            .as_ref()
            .is_some_and(CodexAuth::is_external_chatgpt_tokens)
        {
            UnauthorizedRecoveryMode::External
        } else if matches!(cached_auth.as_ref(), Some(CodexAuth::AnthropicOAuth(_))) {
            UnauthorizedRecoveryMode::AnthropicOAuth
        } else {
            UnauthorizedRecoveryMode::Managed
        };
        let step = match mode {
            UnauthorizedRecoveryMode::Managed => UnauthorizedRecoveryStep::Reload,
            UnauthorizedRecoveryMode::External => UnauthorizedRecoveryStep::ExternalRefresh,
            UnauthorizedRecoveryMode::AnthropicOAuth => UnauthorizedRecoveryStep::Reload,
        };
        Self {
            manager,
            step,
            expected_account_id,
            mode,
        }
    }

    pub fn has_next(&self) -> bool {
        let auth = self.manager.auth_cached();
        let is_recoverable = auth
            .as_ref()
            .is_some_and(|a| a.is_chatgpt_auth() || matches!(a, CodexAuth::AnthropicOAuth(_)));
        if !is_recoverable {
            return false;
        }

        if self.mode == UnauthorizedRecoveryMode::External
            && !self.manager.has_external_auth_refresher()
        {
            return false;
        }

        !matches!(self.step, UnauthorizedRecoveryStep::Done)
    }

    pub fn unavailable_reason(&self) -> &'static str {
        let auth = self.manager.auth_cached();
        let is_recoverable = auth
            .as_ref()
            .is_some_and(|a| a.is_chatgpt_auth() || matches!(a, CodexAuth::AnthropicOAuth(_)));
        if !is_recoverable {
            return "not_recoverable_auth";
        }

        if self.mode == UnauthorizedRecoveryMode::External
            && !self.manager.has_external_auth_refresher()
        {
            return "no_external_refresher";
        }

        if matches!(self.step, UnauthorizedRecoveryStep::Done) {
            return "recovery_exhausted";
        }

        "ready"
    }

    pub fn mode_name(&self) -> &'static str {
        match self.mode {
            UnauthorizedRecoveryMode::Managed => "managed",
            UnauthorizedRecoveryMode::External => "external",
            UnauthorizedRecoveryMode::AnthropicOAuth => "anthropic_oauth",
        }
    }

    pub fn step_name(&self) -> &'static str {
        match self.step {
            UnauthorizedRecoveryStep::Reload => "reload",
            UnauthorizedRecoveryStep::RefreshToken => "refresh_token",
            UnauthorizedRecoveryStep::ExternalRefresh => "external_refresh",
            UnauthorizedRecoveryStep::Done => "done",
        }
    }

    pub async fn next(&mut self) -> Result<UnauthorizedRecoveryStepResult, RefreshTokenError> {
        if !self.has_next() {
            return Err(RefreshTokenError::Permanent(RefreshTokenFailedError::new(
                RefreshTokenFailedReason::Other,
                "No more recovery steps available.",
            )));
        }

        // Anthropic OAuth has its own recovery path — skip account-id guard
        if self.mode == UnauthorizedRecoveryMode::AnthropicOAuth {
            return self.next_anthropic_oauth().await;
        }

        match self.step {
            UnauthorizedRecoveryStep::Reload => {
                match self
                    .manager
                    .reload_if_account_id_matches(self.expected_account_id.as_deref())
                {
                    ReloadOutcome::ReloadedChanged => {
                        self.step = UnauthorizedRecoveryStep::RefreshToken;
                        return Ok(UnauthorizedRecoveryStepResult {
                            auth_state_changed: Some(true),
                        });
                    }
                    ReloadOutcome::ReloadedNoChange => {
                        self.step = UnauthorizedRecoveryStep::RefreshToken;
                        return Ok(UnauthorizedRecoveryStepResult {
                            auth_state_changed: Some(false),
                        });
                    }
                    ReloadOutcome::Skipped => {
                        self.step = UnauthorizedRecoveryStep::Done;
                        return Err(RefreshTokenError::Permanent(RefreshTokenFailedError::new(
                            RefreshTokenFailedReason::Other,
                            REFRESH_TOKEN_ACCOUNT_MISMATCH_MESSAGE.to_string(),
                        )));
                    }
                }
            }
            UnauthorizedRecoveryStep::RefreshToken => {
                self.manager.refresh_token_from_authority().await?;
                self.step = UnauthorizedRecoveryStep::Done;
                return Ok(UnauthorizedRecoveryStepResult {
                    auth_state_changed: Some(true),
                });
            }
            UnauthorizedRecoveryStep::ExternalRefresh => {
                self.manager
                    .refresh_external_auth(ExternalAuthRefreshReason::Unauthorized)
                    .await?;
                self.step = UnauthorizedRecoveryStep::Done;
                return Ok(UnauthorizedRecoveryStepResult {
                    auth_state_changed: Some(true),
                });
            }
            UnauthorizedRecoveryStep::Done => {}
        }
        Ok(UnauthorizedRecoveryStepResult {
            auth_state_changed: None,
        })
    }

    /// Anthropic OAuth recovery: reload from storage, then force-refresh.
    /// No account-id guard — Anthropic doesn't use account IDs.
    async fn next_anthropic_oauth(
        &mut self,
    ) -> Result<UnauthorizedRecoveryStepResult, RefreshTokenError> {
        match self.step {
            UnauthorizedRecoveryStep::Reload => {
                // Reload from storage — another process may have refreshed the token.
                let old_auth = self.manager.auth_cached();
                self.manager.reload();
                let new_auth = self.manager.auth_cached();
                let changed = !AuthManager::auths_equal(old_auth.as_ref(), new_auth.as_ref());
                self.step = UnauthorizedRecoveryStep::RefreshToken;
                Ok(UnauthorizedRecoveryStepResult {
                    auth_state_changed: Some(changed),
                })
            }
            UnauthorizedRecoveryStep::RefreshToken => {
                // Force refresh — we got a 401, the token is definitely bad.
                self.manager.force_refresh_anthropic_oauth().await?;
                self.step = UnauthorizedRecoveryStep::Done;
                Ok(UnauthorizedRecoveryStepResult {
                    auth_state_changed: Some(true),
                })
            }
            _ => {
                self.step = UnauthorizedRecoveryStep::Done;
                Ok(UnauthorizedRecoveryStepResult {
                    auth_state_changed: None,
                })
            }
        }
    }
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

    fn reload_if_account_id_matches(&self, expected_account_id: Option<&str>) -> ReloadOutcome {
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

    fn auths_equal(a: Option<&CodexAuth>, b: Option<&CodexAuth>) -> bool {
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

    async fn refresh_external_auth(
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

#[cfg(test)]
#[path = "auth_tests.rs"]
mod tests;
