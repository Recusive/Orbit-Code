use super::*;
use crate::token_data::IdTokenInfo;
use anyhow::Context;
use base64::Engine;
use pretty_assertions::assert_eq;
use serde_json::json;
use tempfile::tempdir;

use keyring::Error as KeyringError;
use orbit_code_keyring_store::tests::MockKeyringStore;

/// Helper: build a v2 auth with an OpenAI API key.
fn v2_api_key(key: &str) -> AuthDotJsonV2 {
    let mut v2 = AuthDotJsonV2::new();
    v2.set_provider_auth(
        ProviderName::OpenAI,
        ProviderAuth::OpenAiApiKey {
            key: key.to_string(),
        },
    );
    v2
}

/// Helper: build a v2 auth with ChatGPT tokens.
fn v2_chatgpt(prefix: &str) -> AuthDotJsonV2 {
    let mut v2 = AuthDotJsonV2::new();
    v2.set_provider_auth(
        ProviderName::OpenAI,
        ProviderAuth::Chatgpt {
            tokens: TokenData {
                id_token: id_token_with_prefix(prefix),
                access_token: format!("{prefix}-access"),
                refresh_token: format!("{prefix}-refresh"),
                account_id: Some(format!("{prefix}-account-id")),
            },
            last_refresh: None,
        },
    );
    v2
}

#[tokio::test]
async fn file_storage_load_returns_v2() -> anyhow::Result<()> {
    let orbit_code_home = tempdir()?;
    let storage = FileAuthStorage::new(orbit_code_home.path().to_path_buf());
    let v2 = v2_api_key("test-key");

    storage.save(&v2).context("failed to save auth file")?;

    let loaded = storage.load().context("failed to load auth file")?;
    assert_eq!(Some(v2), loaded);
    Ok(())
}

#[tokio::test]
async fn file_storage_save_persists_v2() -> anyhow::Result<()> {
    let orbit_code_home = tempdir()?;
    let storage = FileAuthStorage::new(orbit_code_home.path().to_path_buf());
    let v2 = v2_api_key("test-key");

    storage.save(&v2).context("failed to save auth file")?;

    let loaded = storage.load().context("failed to load auth file")?;
    assert_eq!(Some(v2), loaded);
    Ok(())
}

#[test]
fn file_storage_delete_removes_auth_file() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let v2 = v2_api_key("sk-test-key");
    let storage = create_auth_storage(dir.path().to_path_buf(), AuthCredentialsStoreMode::File);
    storage.save(&v2)?;
    assert!(dir.path().join("auth.json").exists());
    let storage = FileAuthStorage::new(dir.path().to_path_buf());
    let removed = storage.delete()?;
    assert!(removed);
    assert!(!dir.path().join("auth.json").exists());
    Ok(())
}

#[test]
fn ephemeral_storage_save_load_delete_is_in_memory_only() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let storage = create_auth_storage(
        dir.path().to_path_buf(),
        AuthCredentialsStoreMode::Ephemeral,
    );
    let v2 = v2_api_key("sk-ephemeral");

    storage.save(&v2)?;
    let loaded = storage.load()?;
    assert_eq!(Some(v2), loaded);

    let removed = storage.delete()?;
    assert!(removed);
    let loaded = storage.load()?;
    assert_eq!(None, loaded);
    assert!(!get_auth_file(dir.path()).exists());
    Ok(())
}

#[test]
fn file_storage_loads_v1_format_and_converts() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let auth_file = get_auth_file(dir.path());
    // Write a v1-format file directly
    let v1_json = json!({
        "auth_mode": "apikey",
        "OPENAI_API_KEY": "sk-legacy-key"
    });
    std::fs::create_dir_all(dir.path())?;
    std::fs::write(&auth_file, serde_json::to_string_pretty(&v1_json)?)?;

    let storage = FileAuthStorage::new(dir.path().to_path_buf());
    let loaded = storage.load()?.expect("should load v1 file");

    // Should auto-migrate to v2 with OpenAI API key
    assert_eq!(loaded.version, 2);
    assert_eq!(
        loaded.provider_auth(ProviderName::OpenAI),
        Some(&ProviderAuth::OpenAiApiKey {
            key: "sk-legacy-key".to_string()
        })
    );
    Ok(())
}

#[test]
fn v1_to_v2_migration_preserves_chatgpt_auth() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let auth_file = get_auth_file(dir.path());

    // Build a valid v1 ChatGPT auth using the proper JWT helper
    let id_token = id_token_with_prefix("chatgpt");
    let tokens = TokenData {
        id_token,
        access_token: "chatgpt-access".to_string(),
        refresh_token: "chatgpt-refresh".to_string(),
        account_id: Some("acc-123".to_string()),
    };
    let v1 = AuthDotJson {
        auth_mode: Some(AuthMode::Chatgpt),
        openai_api_key: None,
        tokens: Some(tokens),
        last_refresh: Some(Utc::now()),
    };

    std::fs::create_dir_all(dir.path())?;
    std::fs::write(&auth_file, serde_json::to_string_pretty(&v1)?)?;

    let storage = FileAuthStorage::new(dir.path().to_path_buf());
    let loaded = storage.load()?.expect("should load v1 file");
    assert_eq!(loaded.version, 2);
    assert!(matches!(
        loaded.provider_auth(ProviderName::OpenAI),
        Some(ProviderAuth::Chatgpt { .. })
    ));
    Ok(())
}

#[test]
fn v2_roundtrip() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let storage = FileAuthStorage::new(dir.path().to_path_buf());
    let mut v2 = AuthDotJsonV2::new();
    v2.set_provider_auth(
        ProviderName::OpenAI,
        ProviderAuth::OpenAiApiKey {
            key: "sk-openai".to_string(),
        },
    );
    v2.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicApiKey {
            key: "sk-ant-123".to_string(),
        },
    );

    storage.save(&v2)?;
    let loaded = storage.load()?.expect("should load v2");
    assert_eq!(v2, loaded);
    Ok(())
}

#[test]
fn delete_provider_preserves_other_providers() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let storage = FileAuthStorage::new(dir.path().to_path_buf());
    let mut v2 = AuthDotJsonV2::new();
    v2.set_provider_auth(
        ProviderName::OpenAI,
        ProviderAuth::OpenAiApiKey {
            key: "sk-openai".to_string(),
        },
    );
    v2.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicApiKey {
            key: "sk-ant-123".to_string(),
        },
    );
    storage.save(&v2)?;

    let removed = storage.delete_provider(ProviderName::Anthropic)?;
    assert!(removed);

    let loaded = storage.load()?.expect("should still have OpenAI");
    assert!(loaded.provider_auth(ProviderName::Anthropic).is_none());
    assert!(loaded.provider_auth(ProviderName::OpenAI).is_some());
    Ok(())
}

#[test]
fn delete_provider_on_last_removes_file() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let storage = FileAuthStorage::new(dir.path().to_path_buf());
    let v2 = v2_api_key("sk-only");
    storage.save(&v2)?;

    let removed = storage.delete_provider(ProviderName::OpenAI)?;
    assert!(removed);
    assert!(!get_auth_file(dir.path()).exists());
    Ok(())
}

#[test]
fn empty_providers_has_any_auth_false() {
    let v2 = AuthDotJsonV2::new();
    assert!(!v2.has_any_auth());
}

#[test]
fn provider_name_serde_lowercase() -> anyhow::Result<()> {
    let serialized = serde_json::to_string(&ProviderName::OpenAI)?;
    assert_eq!(serialized, "\"openai\"");
    let serialized = serde_json::to_string(&ProviderName::Anthropic)?;
    assert_eq!(serialized, "\"anthropic\"");
    let deserialized: ProviderName = serde_json::from_str("\"anthropic\"")?;
    assert_eq!(deserialized, ProviderName::Anthropic);
    Ok(())
}

#[test]
fn v1_backup_created_on_first_v2_write() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let auth_file = get_auth_file(dir.path());
    let backup_file = dir.path().join("auth.v1.json.bak");

    // Write a v1 file first
    let v1_json = json!({
        "auth_mode": "apikey",
        "OPENAI_API_KEY": "sk-old"
    });
    std::fs::create_dir_all(dir.path())?;
    std::fs::write(&auth_file, serde_json::to_string_pretty(&v1_json)?)?;

    // Save v2 — should create backup
    let storage = FileAuthStorage::new(dir.path().to_path_buf());
    let v2 = v2_api_key("sk-new");
    storage.save(&v2)?;

    assert!(backup_file.exists(), "v1 backup should be created");

    // Second save should NOT overwrite backup
    let v2_2 = v2_api_key("sk-newer");
    storage.save(&v2_2)?;
    let backup_content = std::fs::read_to_string(&backup_file)?;
    assert!(
        backup_content.contains("sk-old"),
        "backup should preserve original v1 content"
    );
    Ok(())
}

fn seed_keyring_and_fallback_auth_file_for_delete<F>(
    mock_keyring: &MockKeyringStore,
    orbit_code_home: &Path,
    compute_key: F,
) -> anyhow::Result<(String, PathBuf)>
where
    F: FnOnce() -> std::io::Result<String>,
{
    let key = compute_key()?;
    // Store a minimal v2 JSON in the keyring
    let v2 = v2_api_key("keyring-key");
    let serialized = serde_json::to_string(&v2)?;
    mock_keyring.save(KEYRING_SERVICE, &key, &serialized)?;
    let auth_file = get_auth_file(orbit_code_home);
    std::fs::write(&auth_file, "stale")?;
    Ok((key, auth_file))
}

fn seed_keyring_with_v2<F>(
    mock_keyring: &MockKeyringStore,
    compute_key: F,
    auth: &AuthDotJsonV2,
) -> anyhow::Result<()>
where
    F: FnOnce() -> std::io::Result<String>,
{
    let key = compute_key()?;
    let serialized = serde_json::to_string(auth)?;
    mock_keyring.save(KEYRING_SERVICE, &key, &serialized)?;
    Ok(())
}

fn assert_keyring_saved_v2_and_removed_fallback(
    mock_keyring: &MockKeyringStore,
    key: &str,
    orbit_code_home: &Path,
    expected: &AuthDotJsonV2,
) {
    let saved_value = mock_keyring
        .saved_value(key)
        .expect("keyring entry should exist");
    let expected_serialized = serde_json::to_string(expected).expect("serialize expected auth");
    assert_eq!(saved_value, expected_serialized);
    let auth_file = get_auth_file(orbit_code_home);
    assert!(
        !auth_file.exists(),
        "fallback auth.json should be removed after keyring save"
    );
}

fn id_token_with_prefix(prefix: &str) -> IdTokenInfo {
    #[derive(Serialize)]
    struct Header {
        alg: &'static str,
        typ: &'static str,
    }

    let header = Header {
        alg: "none",
        typ: "JWT",
    };
    let payload = json!({
        "email": format!("{prefix}@example.com"),
        "https://api.openai.com/auth": {
            "chatgpt_account_id": format!("{prefix}-account"),
        },
    });
    let encode = |bytes: &[u8]| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
    let header_b64 = encode(&serde_json::to_vec(&header).expect("serialize header"));
    let payload_b64 = encode(&serde_json::to_vec(&payload).expect("serialize payload"));
    let signature_b64 = encode(b"sig");
    let fake_jwt = format!("{header_b64}.{payload_b64}.{signature_b64}");

    crate::token_data::parse_chatgpt_jwt_claims(&fake_jwt).expect("fake JWT should parse")
}

#[test]
fn keyring_auth_storage_load_returns_deserialized_v2() -> anyhow::Result<()> {
    let orbit_code_home = tempdir()?;
    let mock_keyring = MockKeyringStore::default();
    let storage = KeyringAuthStorage::new(
        orbit_code_home.path().to_path_buf(),
        Arc::new(mock_keyring.clone()),
    );
    let expected = v2_api_key("sk-test");
    seed_keyring_with_v2(
        &mock_keyring,
        || compute_store_key(orbit_code_home.path()),
        &expected,
    )?;

    let loaded = storage.load()?;
    assert_eq!(Some(expected), loaded);
    Ok(())
}

#[test]
fn keyring_auth_storage_compute_store_key_for_home_directory() -> anyhow::Result<()> {
    let orbit_code_home = PathBuf::from("~/.codex");

    let key = compute_store_key(orbit_code_home.as_path())?;

    assert_eq!(key, "cli|940db7b1d0e4eb40");
    Ok(())
}

#[test]
fn keyring_auth_storage_load_falls_back_to_legacy_service_and_default_codex_path()
-> anyhow::Result<()> {
    let parent_home = tempdir()?;
    let orbit_code_home = parent_home.path().join(".orbit");
    std::fs::create_dir_all(&orbit_code_home)?;
    let legacy_codex_home = parent_home.path().join(".codex");

    let mock_keyring = MockKeyringStore::default();
    let storage = KeyringAuthStorage::new(orbit_code_home, Arc::new(mock_keyring.clone()));
    let expected = v2_chatgpt("legacy-service");
    let key = compute_store_key(&legacy_codex_home)?;
    let serialized = serde_json::to_string(&expected)?;
    mock_keyring.save(LEGACY_KEYRING_SERVICE, &key, &serialized)?;

    let loaded = storage.load()?;
    assert_eq!(Some(expected), loaded);
    Ok(())
}

#[test]
fn keyring_auth_storage_save_persists_and_removes_fallback_file() -> anyhow::Result<()> {
    let orbit_code_home = tempdir()?;
    let mock_keyring = MockKeyringStore::default();
    let storage = KeyringAuthStorage::new(
        orbit_code_home.path().to_path_buf(),
        Arc::new(mock_keyring.clone()),
    );
    let auth_file = get_auth_file(orbit_code_home.path());
    std::fs::write(&auth_file, "stale")?;
    let v2 = v2_chatgpt("chatgpt");

    storage.save(&v2)?;

    let key = compute_store_key(orbit_code_home.path())?;
    assert_keyring_saved_v2_and_removed_fallback(&mock_keyring, &key, orbit_code_home.path(), &v2);
    Ok(())
}

#[test]
fn keyring_auth_storage_delete_removes_keyring_and_file() -> anyhow::Result<()> {
    let orbit_code_home = tempdir()?;
    let mock_keyring = MockKeyringStore::default();
    let storage = KeyringAuthStorage::new(
        orbit_code_home.path().to_path_buf(),
        Arc::new(mock_keyring.clone()),
    );
    let (key, auth_file) = seed_keyring_and_fallback_auth_file_for_delete(
        &mock_keyring,
        orbit_code_home.path(),
        || compute_store_key(orbit_code_home.path()),
    )?;

    let removed = storage.delete()?;

    assert!(removed, "delete should report removal");
    assert!(
        !mock_keyring.contains(&key),
        "keyring entry should be removed"
    );
    assert!(
        !auth_file.exists(),
        "fallback auth.json should be removed after keyring delete"
    );
    Ok(())
}

#[test]
fn auto_auth_storage_load_prefers_keyring_value() -> anyhow::Result<()> {
    let orbit_code_home = tempdir()?;
    let mock_keyring = MockKeyringStore::default();
    let storage = AutoAuthStorage::new(
        orbit_code_home.path().to_path_buf(),
        Arc::new(mock_keyring.clone()),
    );
    let keyring_auth = v2_chatgpt("keyring");
    seed_keyring_with_v2(
        &mock_keyring,
        || compute_store_key(orbit_code_home.path()),
        &keyring_auth,
    )?;

    let file_auth = v2_chatgpt("file");
    storage.file_storage.save(&file_auth)?;

    let loaded = storage.load()?;
    assert_eq!(loaded, Some(keyring_auth));
    Ok(())
}

#[test]
fn auto_auth_storage_load_uses_file_when_keyring_empty() -> anyhow::Result<()> {
    let orbit_code_home = tempdir()?;
    let mock_keyring = MockKeyringStore::default();
    let storage =
        AutoAuthStorage::new(orbit_code_home.path().to_path_buf(), Arc::new(mock_keyring));

    let expected = v2_chatgpt("file-only");
    storage.file_storage.save(&expected)?;

    let loaded = storage.load()?;
    assert_eq!(loaded, Some(expected));
    Ok(())
}

#[test]
fn auto_auth_storage_load_falls_back_when_keyring_errors() -> anyhow::Result<()> {
    let orbit_code_home = tempdir()?;
    let mock_keyring = MockKeyringStore::default();
    let storage = AutoAuthStorage::new(
        orbit_code_home.path().to_path_buf(),
        Arc::new(mock_keyring.clone()),
    );
    let key = compute_store_key(orbit_code_home.path())?;
    mock_keyring.set_error(&key, KeyringError::Invalid("error".into(), "load".into()));

    let expected = v2_chatgpt("fallback");
    storage.file_storage.save(&expected)?;

    let loaded = storage.load()?;
    assert_eq!(loaded, Some(expected));
    Ok(())
}

#[test]
fn auto_auth_storage_save_prefers_keyring() -> anyhow::Result<()> {
    let orbit_code_home = tempdir()?;
    let mock_keyring = MockKeyringStore::default();
    let storage = AutoAuthStorage::new(
        orbit_code_home.path().to_path_buf(),
        Arc::new(mock_keyring.clone()),
    );
    let key = compute_store_key(orbit_code_home.path())?;

    let stale = v2_chatgpt("stale");
    storage.file_storage.save(&stale)?;

    let expected = v2_chatgpt("to-save");
    storage.save(&expected)?;

    assert_keyring_saved_v2_and_removed_fallback(
        &mock_keyring,
        &key,
        orbit_code_home.path(),
        &expected,
    );
    Ok(())
}

#[test]
fn auto_auth_storage_save_falls_back_when_keyring_errors() -> anyhow::Result<()> {
    let orbit_code_home = tempdir()?;
    let mock_keyring = MockKeyringStore::default();
    let storage = AutoAuthStorage::new(
        orbit_code_home.path().to_path_buf(),
        Arc::new(mock_keyring.clone()),
    );
    let key = compute_store_key(orbit_code_home.path())?;
    mock_keyring.set_error(&key, KeyringError::Invalid("error".into(), "save".into()));

    let auth = v2_chatgpt("fallback");
    storage.save(&auth)?;

    let auth_file = get_auth_file(orbit_code_home.path());
    assert!(
        auth_file.exists(),
        "fallback auth.json should be created when keyring save fails"
    );
    let saved = storage
        .file_storage
        .load()?
        .context("fallback auth should exist")?;
    assert_eq!(saved, auth);
    assert!(
        mock_keyring.saved_value(&key).is_none(),
        "keyring should not contain value when save fails"
    );
    Ok(())
}

#[test]
fn auto_auth_storage_delete_removes_keyring_and_file() -> anyhow::Result<()> {
    let orbit_code_home = tempdir()?;
    let mock_keyring = MockKeyringStore::default();
    let storage = AutoAuthStorage::new(
        orbit_code_home.path().to_path_buf(),
        Arc::new(mock_keyring.clone()),
    );
    let (key, auth_file) = seed_keyring_and_fallback_auth_file_for_delete(
        &mock_keyring,
        orbit_code_home.path(),
        || compute_store_key(orbit_code_home.path()),
    )?;

    let removed = storage.delete()?;

    assert!(removed, "delete should report removal");
    assert!(
        !mock_keyring.contains(&key),
        "keyring entry should be removed"
    );
    assert!(
        !auth_file.exists(),
        "fallback auth.json should be removed after delete"
    );
    Ok(())
}

#[test]
fn keyring_loads_v1_and_converts_to_v2() -> anyhow::Result<()> {
    let orbit_code_home = tempdir()?;
    let mock_keyring = MockKeyringStore::default();
    let storage = KeyringAuthStorage::new(
        orbit_code_home.path().to_path_buf(),
        Arc::new(mock_keyring.clone()),
    );
    // Store v1 format JSON in keyring
    let v1_json = json!({
        "auth_mode": "apikey",
        "OPENAI_API_KEY": "sk-from-keyring"
    });
    let key = compute_store_key(orbit_code_home.path())?;
    mock_keyring.save(KEYRING_SERVICE, &key, &serde_json::to_string(&v1_json)?)?;

    let loaded = storage.load()?.expect("should load from keyring");
    assert_eq!(loaded.version, 2);
    assert_eq!(
        loaded.provider_auth(ProviderName::OpenAI),
        Some(&ProviderAuth::OpenAiApiKey {
            key: "sk-from-keyring".to_string()
        })
    );
    Ok(())
}

#[test]
fn to_v1_openai_roundtrip() -> anyhow::Result<()> {
    // v1 → v2 → v1 should preserve OpenAI fields
    let v1 = AuthDotJson {
        auth_mode: Some(AuthMode::ApiKey),
        openai_api_key: Some("sk-test".to_string()),
        tokens: None,
        last_refresh: None,
    };
    let v2 = AuthDotJsonV2::from(v1.clone());
    let back_to_v1 = v2.to_v1_openai();
    assert_eq!(v1, back_to_v1);
    Ok(())
}
