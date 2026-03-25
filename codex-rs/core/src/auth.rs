//! Authentication types, constants, and module re-exports.
//!
//! This module defines the core authentication types ([`CodexAuth`],
//! [`AuthMode`], [`ChatgptAuth`], etc.), environment variable constants,
//! and the ChatGPT OAuth token refresh primitives. Sub-modules handle
//! persistence ([`persistence`]), the unauthorized-recovery state machine
//! ([`recovery`]), the central [`AuthManager`](manager::AuthManager), and
//! credential storage backends ([`storage`]).

pub(crate) mod manager;
pub(crate) mod persistence;
mod recovery;
mod storage;

use async_trait::async_trait;
use chrono::Utc;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use orbit_code_app_server_protocol::AuthMode as ApiAuthMode;
use orbit_code_otel::TelemetryAuthMode;

use crate::error::RefreshTokenFailedError;
use crate::error::RefreshTokenFailedReason;
use crate::token_data::KnownPlan as InternalKnownPlan;
use crate::token_data::PlanType as InternalPlanType;
use crate::token_data::TokenData;
use crate::token_data::parse_chatgpt_jwt_claims;
use orbit_code_client::CodexHttpClient;
use orbit_code_protocol::account::PlanType as AccountPlanType;
use thiserror::Error;

// ── Re-exports from storage ──────────────────────────────────────────
pub use crate::auth::storage::AuthCredentialsStoreMode;
pub use crate::auth::storage::AuthDotJson;
pub use crate::auth::storage::AuthDotJsonV2;
pub(crate) use crate::auth::storage::AuthStorageBackend;
pub use crate::auth::storage::ProviderAuth;
pub use crate::auth::storage::ProviderName;
pub(crate) use crate::auth::storage::create_auth_storage;

// ── Re-exports from persistence ──────────────────────────────────────
pub use persistence::enforce_login_restrictions;
pub use persistence::load_auth_dot_json;
pub use persistence::load_auth_dot_json_v2;
pub use persistence::login_with_api_key;
pub use persistence::login_with_chatgpt_auth_tokens;
pub use persistence::logout;
pub use persistence::logout_provider;
pub use persistence::save_auth;
pub use persistence::save_auth_v2;

// ── Re-exports from persistence (test-only) ─────────────────────────
// These are `pub(crate)` in persistence.rs but only used via `super::*`
// from the test module.
#[cfg(test)]
use persistence::load_auth;
#[cfg(test)]
use persistence::persist_tokens;
#[cfg(test)]
use recovery::UnauthorizedRecoveryMode;
#[cfg(test)]
use recovery::UnauthorizedRecoveryStep;
#[cfg(test)]
use serial_test::serial;

// ── Re-exports from manager ─────────────────────────────────────────
pub use manager::AuthManager;

// ── Re-exports from recovery ────────────────────────────────────────
pub use recovery::UnauthorizedRecovery;
pub use recovery::UnauthorizedRecoveryStepResult;

// ── Re-exports from anthropic_auth ──────────────────────────────────
pub use crate::anthropic_auth::AnthropicApiKeyAuth;
pub use crate::anthropic_auth::AnthropicOAuthAuth;

// ── AuthMode ────────────────────────────────────────────────────────

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

// ── CodexAuth & supporting types ────────────────────────────────────

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

// ── RefreshTokenError ───────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum RefreshTokenError {
    #[error("{0}")]
    Permanent(#[from] RefreshTokenFailedError),
    #[error(transparent)]
    Transient(#[from] std::io::Error),
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

// ── ExternalAuth types ──────────────────────────────────────────────

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

// ── Environment variable constants & helpers ────────────────────────

pub const OPENAI_API_KEY_ENV_VAR: &str = "OPENAI_API_KEY";
pub const ORBIT_API_KEY_ENV_VAR: &str = "ORBIT_API_KEY";

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
}

pub fn read_orbit_code_api_key_from_env() -> Option<String> {
    read_orbit_api_key_from_env()
}

// ── CodexAuth impl ──────────────────────────────────────────────────

impl CodexAuth {
    pub(crate) fn from_auth_dot_json(
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
        persistence::load_auth(
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
    pub(crate) fn get_current_auth_json(&self) -> Option<AuthDotJson> {
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

    pub(crate) fn from_api_key_with_client(api_key: &str, _client: CodexHttpClient) -> Self {
        Self::ApiKey(ApiKeyAuth {
            api_key: api_key.to_owned(),
        })
    }

    pub fn from_api_key(api_key: &str) -> Self {
        Self::from_api_key_with_client(api_key, crate::default_client::create_client())
    }
}

// ── ChatgptAuth helpers (used by manager) ───────────────────────────

impl ChatgptAuth {
    pub(crate) fn current_auth_json(&self) -> Option<AuthDotJson> {
        #[expect(clippy::unwrap_used)]
        self.state.auth_dot_json.lock().unwrap().clone()
    }

    pub(crate) fn current_token_data(&self) -> Option<TokenData> {
        self.current_auth_json().and_then(|auth| auth.tokens)
    }

    pub(crate) fn storage(&self) -> &Arc<dyn AuthStorageBackend> {
        &self.storage
    }

    pub(crate) fn client(&self) -> &CodexHttpClient {
        &self.state.client
    }
}

// ── AuthDotJson helpers ─────────────────────────────────────────────

impl AuthDotJson {
    pub(crate) fn from_external_tokens(external: &ExternalAuthTokens) -> std::io::Result<Self> {
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

    pub(crate) fn resolved_mode(&self) -> ApiAuthMode {
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

// Shared constant for token refresh (client id used for oauth token refresh flow)
pub const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
#[path = "auth_tests.rs"]
mod tests;
