use super::*;
use crate::auth::storage::FileAuthStorage;
use crate::auth::storage::get_auth_file;
use crate::config::Config;
use crate::config::ConfigBuilder;
use crate::token_data::IdTokenInfo;
use crate::token_data::KnownPlan as InternalKnownPlan;
use crate::token_data::PlanType as InternalPlanType;
use orbit_code_protocol::account::PlanType as AccountPlanType;

use base64::Engine;
use orbit_code_protocol::config_types::ForcedLoginMethod;
use pretty_assertions::assert_eq;
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn refresh_without_id_token() {
    let orbit_code_home = tempdir().unwrap();
    let fake_jwt = write_auth_file(
        AuthFileParams {
            openai_api_key: None,
            chatgpt_plan_type: Some("pro".to_string()),
            chatgpt_account_id: None,
        },
        orbit_code_home.path(),
    )
    .expect("failed to write auth file");

    let storage = create_auth_storage(
        orbit_code_home.path().to_path_buf(),
        AuthCredentialsStoreMode::File,
    );
    let updated = super::persist_tokens(
        &storage,
        None,
        Some("new-access-token".to_string()),
        Some("new-refresh-token".to_string()),
    )
    .expect("update_tokens should succeed");

    let tokens = updated.tokens.expect("tokens should exist");
    assert_eq!(tokens.id_token.raw_jwt, fake_jwt);
    assert_eq!(tokens.access_token, "new-access-token");
    assert_eq!(tokens.refresh_token, "new-refresh-token");
}

#[test]
fn login_with_api_key_overwrites_existing_auth_json() {
    let dir = tempdir().unwrap();
    let auth_path = dir.path().join("auth.json");
    let stale_auth = json!({
        "OPENAI_API_KEY": "sk-old",
        "tokens": {
            "id_token": "stale.header.payload",
            "access_token": "stale-access",
            "refresh_token": "stale-refresh",
            "account_id": "stale-acc"
        }
    });
    std::fs::write(
        &auth_path,
        serde_json::to_string_pretty(&stale_auth).unwrap(),
    )
    .unwrap();

    super::login_with_api_key(dir.path(), "sk-new", AuthCredentialsStoreMode::File)
        .expect("login_with_api_key should succeed");

    let storage = FileAuthStorage::new(dir.path().to_path_buf());
    let v2 = storage
        .load()
        .expect("auth.json should load")
        .expect("auth.json should exist");
    let auth = v2.to_v1_openai();
    assert_eq!(auth.openai_api_key.as_deref(), Some("sk-new"));
    assert!(auth.tokens.is_none(), "tokens should be cleared");
}

#[test]
fn missing_auth_json_returns_none() {
    let dir = tempdir().unwrap();
    let auth = CodexAuth::from_auth_storage(dir.path(), AuthCredentialsStoreMode::File)
        .expect("call should succeed");
    assert_eq!(auth, None);
}

#[tokio::test]
#[serial(orbit_code_api_key)]
async fn pro_account_with_no_api_key_uses_chatgpt_auth() {
    let orbit_code_home = tempdir().unwrap();
    let fake_jwt = write_auth_file(
        AuthFileParams {
            openai_api_key: None,
            chatgpt_plan_type: Some("pro".to_string()),
            chatgpt_account_id: None,
        },
        orbit_code_home.path(),
    )
    .expect("failed to write auth file");

    let auth = super::load_auth(
        orbit_code_home.path(),
        false,
        AuthCredentialsStoreMode::File,
    )
    .unwrap()
    .unwrap();
    assert_eq!(None, auth.api_key());
    assert_eq!(AuthMode::Chatgpt, auth.auth_mode());
    assert_eq!(auth.get_chatgpt_user_id().as_deref(), Some("user-12345"));

    let auth_dot_json = auth
        .get_current_auth_json()
        .expect("AuthDotJson should exist");
    let last_refresh = auth_dot_json
        .last_refresh
        .expect("last_refresh should be recorded");

    // After v1→v2→v1 roundtrip, the auth_mode is normalized to
    // Some(Chatgpt) instead of None (v2 format always has explicit types).
    assert_eq!(
        AuthDotJson {
            auth_mode: Some(ApiAuthMode::Chatgpt),
            openai_api_key: None,
            tokens: Some(TokenData {
                id_token: IdTokenInfo {
                    email: Some("user@example.com".to_string()),
                    chatgpt_plan_type: Some(InternalPlanType::Known(InternalKnownPlan::Pro)),
                    chatgpt_user_id: Some("user-12345".to_string()),
                    chatgpt_account_id: None,
                    raw_jwt: fake_jwt,
                },
                access_token: "test-access-token".to_string(),
                refresh_token: "test-refresh-token".to_string(),
                account_id: None,
            }),
            last_refresh: Some(last_refresh),
        },
        auth_dot_json
    );
}

#[tokio::test]
#[serial(orbit_code_api_key)]
async fn loads_api_key_from_auth_json() {
    let dir = tempdir().unwrap();
    let auth_file = dir.path().join("auth.json");
    std::fs::write(
        auth_file,
        r#"{"OPENAI_API_KEY":"sk-test-key","tokens":null,"last_refresh":null}"#,
    )
    .unwrap();

    let auth = super::load_auth(dir.path(), false, AuthCredentialsStoreMode::File)
        .unwrap()
        .unwrap();
    assert_eq!(auth.auth_mode(), AuthMode::ApiKey);
    assert_eq!(auth.api_key(), Some("sk-test-key"));

    assert!(auth.get_token_data().is_err());
}

#[test]
fn logout_removes_auth_file() -> Result<(), std::io::Error> {
    let dir = tempdir()?;
    let auth_dot_json = AuthDotJson {
        auth_mode: Some(ApiAuthMode::ApiKey),
        openai_api_key: Some("sk-test-key".to_string()),
        tokens: None,
        last_refresh: None,
    };
    super::save_auth(dir.path(), &auth_dot_json, AuthCredentialsStoreMode::File)?;
    let auth_file = get_auth_file(dir.path());
    assert!(auth_file.exists());
    assert!(logout(dir.path(), AuthCredentialsStoreMode::File)?);
    assert!(!auth_file.exists());
    Ok(())
}

#[test]
fn unauthorized_recovery_reports_mode_and_step_names() {
    let dir = tempdir().unwrap();
    let manager = AuthManager::shared(
        dir.path().to_path_buf(),
        false,
        AuthCredentialsStoreMode::File,
    );
    let managed = UnauthorizedRecovery {
        manager: Arc::clone(&manager),
        step: UnauthorizedRecoveryStep::Reload,
        expected_account_id: None,
        mode: UnauthorizedRecoveryMode::Managed,
    };
    assert_eq!(managed.mode_name(), "managed");
    assert_eq!(managed.step_name(), "reload");

    let external = UnauthorizedRecovery {
        manager,
        step: UnauthorizedRecoveryStep::ExternalRefresh,
        expected_account_id: None,
        mode: UnauthorizedRecoveryMode::External,
    };
    assert_eq!(external.mode_name(), "external");
    assert_eq!(external.step_name(), "external_refresh");
}

struct AuthFileParams {
    openai_api_key: Option<String>,
    chatgpt_plan_type: Option<String>,
    chatgpt_account_id: Option<String>,
}

fn write_auth_file(params: AuthFileParams, orbit_code_home: &Path) -> std::io::Result<String> {
    let auth_file = get_auth_file(orbit_code_home);
    // Create a minimal valid JWT for the id_token field.
    #[derive(Serialize)]
    struct Header {
        alg: &'static str,
        typ: &'static str,
    }
    let header = Header {
        alg: "none",
        typ: "JWT",
    };
    let mut auth_payload = serde_json::json!({
        "chatgpt_user_id": "user-12345",
        "user_id": "user-12345",
    });

    if let Some(chatgpt_plan_type) = params.chatgpt_plan_type {
        auth_payload["chatgpt_plan_type"] = serde_json::Value::String(chatgpt_plan_type);
    }

    if let Some(chatgpt_account_id) = params.chatgpt_account_id {
        let org_value = serde_json::Value::String(chatgpt_account_id);
        auth_payload["chatgpt_account_id"] = org_value;
    }

    let payload = serde_json::json!({
        "email": "user@example.com",
        "email_verified": true,
        "https://api.openai.com/auth": auth_payload,
    });
    let b64 = |b: &[u8]| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b);
    let header_b64 = b64(&serde_json::to_vec(&header)?);
    let payload_b64 = b64(&serde_json::to_vec(&payload)?);
    let signature_b64 = b64(b"sig");
    let fake_jwt = format!("{header_b64}.{payload_b64}.{signature_b64}");

    let auth_json_data = json!({
        "OPENAI_API_KEY": params.openai_api_key,
        "tokens": {
            "id_token": fake_jwt,
            "access_token": "test-access-token",
            "refresh_token": "test-refresh-token"
        },
        "last_refresh": Utc::now(),
    });
    let auth_json = serde_json::to_string_pretty(&auth_json_data)?;
    std::fs::write(auth_file, auth_json)?;
    Ok(fake_jwt)
}

async fn build_config(
    orbit_code_home: &Path,
    forced_login_method: Option<ForcedLoginMethod>,
    forced_chatgpt_workspace_id: Option<String>,
) -> Config {
    let mut config = ConfigBuilder::default()
        .orbit_code_home(orbit_code_home.to_path_buf())
        .build()
        .await
        .expect("config should load");
    config.forced_login_method = forced_login_method;
    config.forced_chatgpt_workspace_id = forced_chatgpt_workspace_id;
    config
}

/// Use sparingly.
/// TODO (gpeal): replace this with an injectable env var provider.
#[cfg(test)]
struct EnvVarGuard {
    key: &'static str,
    original: Option<std::ffi::OsString>,
}

#[cfg(test)]
impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let original = env::var_os(key);
        unsafe {
            env::set_var(key, value);
        }
        Self { key, original }
    }
}

#[cfg(test)]
impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.original {
                Some(value) => env::set_var(self.key, value),
                None => env::remove_var(self.key),
            }
        }
    }
}

#[tokio::test]
async fn enforce_login_restrictions_logs_out_for_method_mismatch() {
    let orbit_code_home = tempdir().unwrap();
    login_with_api_key(
        orbit_code_home.path(),
        "sk-test",
        AuthCredentialsStoreMode::File,
    )
    .expect("seed api key");

    let config = build_config(
        orbit_code_home.path(),
        Some(ForcedLoginMethod::Chatgpt),
        None,
    )
    .await;

    let err =
        super::enforce_login_restrictions(&config).expect_err("expected method mismatch to error");
    assert!(err.to_string().contains("ChatGPT login is required"));
    assert!(
        !orbit_code_home.path().join("auth.json").exists(),
        "auth.json should be removed on mismatch"
    );
}

#[tokio::test]
#[serial(orbit_code_api_key)]
async fn enforce_login_restrictions_logs_out_for_workspace_mismatch() {
    let orbit_code_home = tempdir().unwrap();
    let _jwt = write_auth_file(
        AuthFileParams {
            openai_api_key: None,
            chatgpt_plan_type: Some("pro".to_string()),
            chatgpt_account_id: Some("org_another_org".to_string()),
        },
        orbit_code_home.path(),
    )
    .expect("failed to write auth file");

    let config = build_config(orbit_code_home.path(), None, Some("org_mine".to_string())).await;

    let err = super::enforce_login_restrictions(&config)
        .expect_err("expected workspace mismatch to error");
    assert!(err.to_string().contains("workspace org_mine"));
    assert!(
        !orbit_code_home.path().join("auth.json").exists(),
        "auth.json should be removed on mismatch"
    );
}

#[tokio::test]
#[serial(orbit_code_api_key)]
async fn enforce_login_restrictions_allows_matching_workspace() {
    let orbit_code_home = tempdir().unwrap();
    let _jwt = write_auth_file(
        AuthFileParams {
            openai_api_key: None,
            chatgpt_plan_type: Some("pro".to_string()),
            chatgpt_account_id: Some("org_mine".to_string()),
        },
        orbit_code_home.path(),
    )
    .expect("failed to write auth file");

    let config = build_config(orbit_code_home.path(), None, Some("org_mine".to_string())).await;

    super::enforce_login_restrictions(&config).expect("matching workspace should succeed");
    assert!(
        orbit_code_home.path().join("auth.json").exists(),
        "auth.json should remain when restrictions pass"
    );
}

#[tokio::test]
async fn enforce_login_restrictions_allows_api_key_if_login_method_not_set_but_forced_chatgpt_workspace_id_is_set()
 {
    let orbit_code_home = tempdir().unwrap();
    login_with_api_key(
        orbit_code_home.path(),
        "sk-test",
        AuthCredentialsStoreMode::File,
    )
    .expect("seed api key");

    let config = build_config(orbit_code_home.path(), None, Some("org_mine".to_string())).await;

    super::enforce_login_restrictions(&config).expect("matching workspace should succeed");
    assert!(
        orbit_code_home.path().join("auth.json").exists(),
        "auth.json should remain when restrictions pass"
    );
}

#[tokio::test]
#[serial(orbit_code_api_key)]
async fn enforce_login_restrictions_blocks_env_api_key_when_chatgpt_required() {
    let _guard = EnvVarGuard::set(ORBIT_API_KEY_ENV_VAR, "sk-env");
    let orbit_code_home = tempdir().unwrap();

    let config = build_config(
        orbit_code_home.path(),
        Some(ForcedLoginMethod::Chatgpt),
        None,
    )
    .await;

    let err = super::enforce_login_restrictions(&config)
        .expect_err("environment API key should not satisfy forced ChatGPT login");
    assert!(
        err.to_string()
            .contains("ChatGPT login is required, but an API key is currently being used.")
    );
}

#[test]
#[serial(orbit_code_api_key)]
fn read_orbit_api_key_from_env_prefers_orbit_over_legacy_codex_env_var() {
    let _orbit_guard = EnvVarGuard::set(ORBIT_API_KEY_ENV_VAR, "sk-orbit");
    let _legacy_guard = EnvVarGuard::set(LEGACY_CODEX_API_KEY_ENV_VAR, "sk-codex");

    let api_key = super::read_orbit_api_key_from_env();

    assert_eq!(api_key.as_deref(), Some("sk-orbit"));
}

#[test]
#[serial(orbit_code_api_key)]
fn read_orbit_api_key_from_env_falls_back_to_legacy_codex_env_var() {
    let _legacy_guard = EnvVarGuard::set(LEGACY_CODEX_API_KEY_ENV_VAR, "sk-codex");

    let api_key = super::read_orbit_api_key_from_env();

    assert_eq!(api_key.as_deref(), Some("sk-codex"));
}

#[test]
fn plan_type_maps_known_plan() {
    let orbit_code_home = tempdir().unwrap();
    let _jwt = write_auth_file(
        AuthFileParams {
            openai_api_key: None,
            chatgpt_plan_type: Some("pro".to_string()),
            chatgpt_account_id: None,
        },
        orbit_code_home.path(),
    )
    .expect("failed to write auth file");

    let auth = super::load_auth(
        orbit_code_home.path(),
        false,
        AuthCredentialsStoreMode::File,
    )
    .expect("load auth")
    .expect("auth available");

    pretty_assertions::assert_eq!(auth.account_plan_type(), Some(AccountPlanType::Pro));
}

#[test]
fn plan_type_maps_unknown_to_unknown() {
    let orbit_code_home = tempdir().unwrap();
    let _jwt = write_auth_file(
        AuthFileParams {
            openai_api_key: None,
            chatgpt_plan_type: Some("mystery-tier".to_string()),
            chatgpt_account_id: None,
        },
        orbit_code_home.path(),
    )
    .expect("failed to write auth file");

    let auth = super::load_auth(
        orbit_code_home.path(),
        false,
        AuthCredentialsStoreMode::File,
    )
    .expect("load auth")
    .expect("auth available");

    pretty_assertions::assert_eq!(auth.account_plan_type(), Some(AccountPlanType::Unknown));
}

#[test]
fn missing_plan_type_maps_to_unknown() {
    let orbit_code_home = tempdir().unwrap();
    let _jwt = write_auth_file(
        AuthFileParams {
            openai_api_key: None,
            chatgpt_plan_type: None,
            chatgpt_account_id: None,
        },
        orbit_code_home.path(),
    )
    .expect("failed to write auth file");

    let auth = super::load_auth(
        orbit_code_home.path(),
        false,
        AuthCredentialsStoreMode::File,
    )
    .expect("load auth")
    .expect("auth available");

    pretty_assertions::assert_eq!(auth.account_plan_type(), Some(AccountPlanType::Unknown));
}

#[test]
fn auth_cached_for_provider_openai_does_not_return_anthropic() {
    // When the only auth in storage is Anthropic OAuth, asking for OpenAI should return None.
    let dir = tempdir().expect("failed to create tempdir");
    let auth_path = dir.path().join("auth.json");
    let v2 = json!({
        "version": 2,
        "providers": {
            "anthropic": {
                "type": "anthropic_oauth",
                "access_token": "sk-ant-oat01-test",
                "refresh_token": "sk-ant-ort01-test",
                "expires_at": 9999999999_i64
            }
        }
    });
    std::fs::write(
        &auth_path,
        serde_json::to_string(&v2).expect("serialize v2"),
    )
    .expect("write auth file");

    let manager = AuthManager::new(
        dir.path().to_path_buf(),
        /*enable_orbit_code_api_key_env*/ false,
        AuthCredentialsStoreMode::File,
    );

    // OpenAI lookup must NOT return the Anthropic token
    let openai_auth = manager.auth_cached_for_provider(ProviderName::OpenAI);
    assert!(
        openai_auth.is_none(),
        "OpenAI lookup should not return Anthropic auth, got: {openai_auth:?}"
    );

    // Anthropic lookup should still work
    let anthropic_auth = manager.auth_cached_for_provider(ProviderName::Anthropic);
    assert!(
        anthropic_auth.is_some(),
        "Anthropic lookup should find the OAuth token"
    );
    assert!(matches!(anthropic_auth, Some(CodexAuth::AnthropicOAuth(_))));
}

#[test]
fn auth_cached_for_provider_openai_finds_openai_in_v2_storage() {
    let dir = tempdir().expect("failed to create tempdir");
    let auth_path = dir.path().join("auth.json");
    let v2 = json!({
        "version": 2,
        "providers": {
            "anthropic": {
                "type": "anthropic_oauth",
                "access_token": "sk-ant-oat01-test",
                "refresh_token": "sk-ant-ort01-test",
                "expires_at": 9999999999_i64
            },
            "openai": {
                "type": "openai_api_key",
                "key": "sk-openai-test"
            }
        }
    });
    std::fs::write(
        &auth_path,
        serde_json::to_string(&v2).expect("serialize v2"),
    )
    .expect("write auth file");

    let manager = AuthManager::new(
        dir.path().to_path_buf(),
        /*enable_orbit_code_api_key_env*/ false,
        AuthCredentialsStoreMode::File,
    );

    let openai_auth = manager.auth_cached_for_provider(ProviderName::OpenAI);
    assert!(
        openai_auth.is_some(),
        "OpenAI lookup should find OpenAI auth from v2 storage"
    );
}

/// Scenario: User logs in with ChatGPT, then adds Anthropic OAuth.
/// Both providers must survive in storage.
#[test]
fn save_auth_preserves_existing_providers_on_chatgpt_then_anthropic() {
    let dir = tempdir().expect("tempdir");
    let store_mode = AuthCredentialsStoreMode::File;

    // Step 1: Save ChatGPT auth (simulates ChatGPT login)
    let chatgpt_auth = AuthDotJson {
        auth_mode: Some(ApiAuthMode::ApiKey),
        openai_api_key: Some("sk-openai-test".to_string()),
        tokens: None,
        last_refresh: None,
    };
    super::save_auth(dir.path(), &chatgpt_auth, store_mode).expect("save chatgpt");

    // Verify ChatGPT was saved
    let v2 = super::load_auth_dot_json_v2(dir.path(), store_mode)
        .expect("load v2")
        .expect("v2 should exist");
    assert!(
        v2.provider_auth(ProviderName::OpenAI).is_some(),
        "OpenAI should exist after ChatGPT login"
    );

    // Step 2: Save Anthropic OAuth (simulates Anthropic onboarding)
    // This is what the TUI onboarding does: load v2, merge, save_auth_v2
    let mut v2_for_anthropic = super::load_auth_dot_json_v2(dir.path(), store_mode)
        .expect("load v2")
        .unwrap_or_else(AuthDotJsonV2::new);
    v2_for_anthropic.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicOAuth {
            access_token: "sk-ant-oat01-test".to_string(),
            refresh_token: "sk-ant-ort01-test".to_string(),
            expires_at: 9999999999,
        },
    );
    super::save_auth_v2(dir.path(), &v2_for_anthropic, store_mode).expect("save anthropic");

    // Step 3: Verify BOTH providers exist
    let final_v2 = super::load_auth_dot_json_v2(dir.path(), store_mode)
        .expect("load final v2")
        .expect("final v2 should exist");
    assert!(
        final_v2.provider_auth(ProviderName::OpenAI).is_some(),
        "OpenAI should STILL exist after Anthropic save"
    );
    assert!(
        final_v2.provider_auth(ProviderName::Anthropic).is_some(),
        "Anthropic should exist after Anthropic save"
    );
}

/// Scenario: User has Anthropic OAuth, then logs in with ChatGPT.
/// Both providers must survive in storage.
#[test]
fn save_auth_preserves_existing_providers_on_anthropic_then_chatgpt() {
    let dir = tempdir().expect("tempdir");
    let store_mode = AuthCredentialsStoreMode::File;

    // Step 1: Write Anthropic OAuth directly to v2
    let mut v2 = AuthDotJsonV2::new();
    v2.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicOAuth {
            access_token: "sk-ant-oat01-test".to_string(),
            refresh_token: "sk-ant-ort01-test".to_string(),
            expires_at: 9999999999,
        },
    );
    super::save_auth_v2(dir.path(), &v2, store_mode).expect("save anthropic");

    // Step 2: Save ChatGPT auth via save_auth (v1 path — the one that was clobbering)
    let chatgpt_auth = AuthDotJson {
        auth_mode: Some(ApiAuthMode::ApiKey),
        openai_api_key: Some("sk-openai-test".to_string()),
        tokens: None,
        last_refresh: None,
    };
    super::save_auth(dir.path(), &chatgpt_auth, store_mode).expect("save chatgpt");

    // Step 3: Verify BOTH providers exist
    let final_v2 = super::load_auth_dot_json_v2(dir.path(), store_mode)
        .expect("load final v2")
        .expect("final v2 should exist");
    assert!(
        final_v2.provider_auth(ProviderName::Anthropic).is_some(),
        "Anthropic should STILL exist after ChatGPT save"
    );
    assert!(
        final_v2.provider_auth(ProviderName::OpenAI).is_some(),
        "OpenAI should exist after ChatGPT save"
    );
}

/// Scenario: save_auth_v2 with only Anthropic should NOT clobber existing OpenAI.
#[test]
fn save_auth_v2_preserves_existing_providers() {
    let dir = tempdir().expect("tempdir");
    let store_mode = AuthCredentialsStoreMode::File;

    // Step 1: Write both providers to storage
    let auth_path = dir.path().join("auth.json");
    let both = json!({
        "version": 2,
        "providers": {
            "openai": {
                "type": "openai_api_key",
                "key": "sk-openai-test"
            },
            "anthropic": {
                "type": "anthropic_oauth",
                "access_token": "sk-ant-oat01-old",
                "refresh_token": "sk-ant-ort01-old",
                "expires_at": 9999999999_i64
            }
        }
    });
    std::fs::write(&auth_path, serde_json::to_string(&both).expect("json")).expect("write");

    // Step 2: save_auth_v2 with ONLY a refreshed Anthropic token
    let mut anthropic_only = AuthDotJsonV2::new();
    anthropic_only.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicOAuth {
            access_token: "sk-ant-oat01-refreshed".to_string(),
            refresh_token: "sk-ant-ort01-refreshed".to_string(),
            expires_at: 9999999999,
        },
    );
    super::save_auth_v2(dir.path(), &anthropic_only, store_mode).expect("save v2");

    // Step 3: Verify OpenAI survived and Anthropic was updated
    let final_v2 = super::load_auth_dot_json_v2(dir.path(), store_mode)
        .expect("load final v2")
        .expect("final v2 should exist");
    assert!(
        final_v2.provider_auth(ProviderName::OpenAI).is_some(),
        "OpenAI should STILL exist after save_auth_v2 with only Anthropic"
    );
    assert!(
        final_v2.provider_auth(ProviderName::Anthropic).is_some(),
        "Anthropic should exist after save_auth_v2"
    );
}
