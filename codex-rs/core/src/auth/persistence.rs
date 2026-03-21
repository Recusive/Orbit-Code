//! Auth persistence, loading, and login/logout operations.
//!
//! This module owns all functions that read from or write to the auth storage
//! backends (file, keyring, ephemeral). Higher-level callers — notably
//! [`AuthManager`](super::manager::AuthManager) — delegate to these functions
//! for storage I/O.

use std::path::Path;
use std::sync::Arc;

use chrono::Utc;
use orbit_code_app_server_protocol::AuthMode as ApiAuthMode;
use orbit_code_protocol::config_types::ForcedLoginMethod;

use crate::anthropic_auth::AnthropicApiKeyAuth;
use crate::auth::AuthMode;
use crate::auth::CodexAuth;
use crate::auth::storage::AuthCredentialsStoreMode;
use crate::auth::storage::AuthDotJson;
use crate::auth::storage::AuthDotJsonV2;
use crate::auth::storage::AuthStorageBackend;
use crate::auth::storage::ProviderAuth;
use crate::auth::storage::ProviderName;
use crate::auth::storage::create_auth_storage;
use crate::config::Config;
use crate::token_data::TokenData;
use crate::token_data::parse_chatgpt_jwt_claims;

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
    // If existing storage is corrupt or unreadable, start fresh.
    let merged = match storage.load() {
        Ok(Some(mut existing)) => {
            for (provider, provider_auth) in &new_v2.providers {
                existing.set_provider_auth(*provider, provider_auth.clone());
            }
            existing
        }
        Ok(None) | Err(_) => new_v2,
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
    let merged = match storage.load() {
        Ok(Some(mut existing)) => {
            for (provider, provider_auth) in &auth.providers {
                existing.set_provider_auth(*provider, provider_auth.clone());
            }
            existing
        }
        Ok(None) | Err(_) => auth.clone(),
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

pub(crate) fn logout_all_stores(
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

pub(crate) fn load_auth(
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
    if enable_orbit_code_api_key_env && let Some(api_key) = super::read_orbit_api_key_from_env() {
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
pub(crate) fn persist_tokens(
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
pub(crate) fn codex_auth_from_provider_auth(provider_auth: &ProviderAuth) -> Option<CodexAuth> {
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
