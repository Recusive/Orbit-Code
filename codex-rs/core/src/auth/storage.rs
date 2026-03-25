use chrono::DateTime;
use chrono::Utc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use tracing::warn;

use crate::token_data::TokenData;
use once_cell::sync::Lazy;
use orbit_code_app_server_protocol::AuthMode;
use orbit_code_keyring_store::DefaultKeyringStore;
use orbit_code_keyring_store::KeyringStore;

/// Determine where Orbit Code should store CLI auth credentials.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AuthCredentialsStoreMode {
    #[default]
    /// Persist credentials in ORBIT_HOME/auth.json.
    File,
    /// Persist credentials in the keyring. Fail if unavailable.
    Keyring,
    /// Use keyring when available; otherwise, fall back to a file in ORBIT_HOME.
    Auto,
    /// Store credentials in memory only for the current process.
    Ephemeral,
}

/// Strongly-typed provider identifier used as HashMap key.
/// Extensible via new variants (e.g., OpenRouter in stage 3c).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderName {
    OpenAI,
    Anthropic,
}

impl fmt::Display for ProviderName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenAI => write!(f, "openai"),
            Self::Anthropic => write!(f, "anthropic"),
        }
    }
}

/// Per-provider auth credential stored in v2 format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ProviderAuth {
    /// OpenAI API key.
    #[serde(rename = "openai_api_key")]
    OpenAiApiKey { key: String },

    /// ChatGPT OAuth (OpenAI-specific, preserves existing token format).
    #[serde(rename = "chatgpt")]
    Chatgpt {
        tokens: TokenData,
        last_refresh: Option<DateTime<Utc>>,
    },

    /// ChatGPT external auth tokens (app-server managed).
    #[serde(rename = "chatgpt_auth_tokens")]
    ChatgptAuthTokens {
        tokens: TokenData,
        last_refresh: Option<DateTime<Utc>>,
    },

    /// Anthropic API key (persisted, not just env var).
    #[serde(rename = "anthropic_api_key")]
    AnthropicApiKey { key: String },

    /// Anthropic OAuth (code-paste flow).
    #[serde(rename = "anthropic_oauth")]
    AnthropicOAuth {
        access_token: String,
        refresh_token: String,
        /// Unix timestamp in SECONDS when the access token expires.
        /// Per CLAUDE.md rule 32: timestamps are integer Unix seconds.
        expires_at: i64,
    },
}

/// V2 auth storage — supports multiple providers side-by-side.
/// This is a SEPARATE type from AuthDotJson (v1) to avoid the fragile hybrid
/// struct problem identified in the audit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDotJsonV2 {
    /// Always 2 for v2 format.
    pub version: u32,

    /// Provider-keyed auth credentials.
    pub providers: HashMap<ProviderName, ProviderAuth>,

    /// Stored-but-inactive credential per provider. When user switches
    /// auth method, the old credential moves here from `providers`.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub alternate_credentials: HashMap<ProviderName, ProviderAuth>,

    /// Last-used auth method per provider. Determines pre-highlight
    /// in the auth popup and credential resolution order.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub preferred_auth_modes: HashMap<ProviderName, AuthMode>,
}

impl AuthDotJsonV2 {
    pub fn new() -> Self {
        Self {
            version: 2,
            providers: HashMap::new(),
            alternate_credentials: HashMap::new(),
            preferred_auth_modes: HashMap::new(),
        }
    }

    /// Get auth for a specific provider.
    pub fn provider_auth(&self, provider: ProviderName) -> Option<&ProviderAuth> {
        self.providers.get(&provider)
    }

    /// Set auth for a specific provider (does not affect other providers).
    ///
    /// When the new credential has a DIFFERENT auth method type than the
    /// existing one (e.g., API key → OAuth), the old credential is
    /// auto-moved to `alternate_credentials`. Same-method rewrites
    /// (e.g., OAuth token refresh, API key rotation) replace in place
    /// without touching the alternate.
    pub fn set_provider_auth(&mut self, provider: ProviderName, auth: ProviderAuth) {
        let new_discriminant = std::mem::discriminant(&auth);
        if let Some(old) = self.providers.insert(provider, auth) {
            // Only preserve as alternate when the auth method type changes.
            // Same-method rewrites (OAuth refresh, key rotation) replace in place.
            if std::mem::discriminant(&old) != new_discriminant {
                self.alternate_credentials.insert(provider, old);
            }
        }
    }

    /// Remove auth for a specific provider.
    pub fn remove_provider_auth(&mut self, provider: ProviderName) -> Option<ProviderAuth> {
        self.providers.remove(&provider)
    }

    /// Restore the alternate credential as active. The current active
    /// credential moves to alternate. Returns true if a swap occurred.
    pub fn restore_alternate_credential(&mut self, provider: ProviderName) -> bool {
        if let Some(alternate) = self.alternate_credentials.remove(&provider) {
            if let Some(current) = self.providers.remove(&provider) {
                self.alternate_credentials.insert(provider, current);
            }
            self.providers.insert(provider, alternate);
            true
        } else {
            false
        }
    }

    /// Remove all credentials for a provider (active + alternate + preference).
    pub fn remove_all_credentials(&mut self, provider: ProviderName) {
        self.providers.remove(&provider);
        self.alternate_credentials.remove(&provider);
        self.preferred_auth_modes.remove(&provider);
    }

    /// Check if any provider has stored credentials.
    pub fn has_any_auth(&self) -> bool {
        !self.providers.is_empty() || !self.alternate_credentials.is_empty()
    }
}

impl Default for AuthDotJsonV2 {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthDotJsonV2 {
    /// Convert v2 back to v1 format (OpenAI provider only).
    /// Used for backward compatibility with code that expects the legacy format.
    pub fn to_v1_openai(&self) -> AuthDotJson {
        match self.provider_auth(ProviderName::OpenAI) {
            Some(ProviderAuth::OpenAiApiKey { key }) => AuthDotJson {
                auth_mode: Some(AuthMode::ApiKey),
                openai_api_key: Some(key.clone()),
                tokens: None,
                last_refresh: None,
            },
            Some(ProviderAuth::Chatgpt {
                tokens,
                last_refresh,
            }) => AuthDotJson {
                auth_mode: Some(AuthMode::Chatgpt),
                openai_api_key: None,
                tokens: Some(tokens.clone()),
                last_refresh: *last_refresh,
            },
            Some(ProviderAuth::ChatgptAuthTokens {
                tokens,
                last_refresh,
            }) => AuthDotJson {
                auth_mode: Some(AuthMode::ChatgptAuthTokens),
                openai_api_key: None,
                tokens: Some(tokens.clone()),
                last_refresh: *last_refresh,
            },
            _ => AuthDotJson {
                auth_mode: None,
                openai_api_key: None,
                tokens: None,
                last_refresh: None,
            },
        }
    }
}

/// Migrate legacy v1 AuthDotJson to v2 format.
/// Called once on load when v1 format is detected.
/// Uses the same `resolved_mode()` logic as the v1 code path to determine
/// the auth mode when `auth_mode` is None.
impl From<AuthDotJson> for AuthDotJsonV2 {
    fn from(v1: AuthDotJson) -> Self {
        let mut v2 = AuthDotJsonV2::new();

        // Mirror resolved_mode(): explicit auth_mode > api_key presence > chatgpt fallback
        let resolved = match v1.auth_mode {
            Some(mode) => mode,
            None if v1.openai_api_key.is_some() => AuthMode::ApiKey,
            None if v1.tokens.is_some() => AuthMode::Chatgpt,
            None => return v2, // Truly empty — no auth stored
        };

        match resolved {
            AuthMode::ApiKey => {
                if let Some(key) = v1.openai_api_key {
                    v2.set_provider_auth(ProviderName::OpenAI, ProviderAuth::OpenAiApiKey { key });
                }
            }
            AuthMode::Chatgpt => {
                if let Some(tokens) = v1.tokens {
                    v2.set_provider_auth(
                        ProviderName::OpenAI,
                        ProviderAuth::Chatgpt {
                            tokens,
                            last_refresh: v1.last_refresh,
                        },
                    );
                }
            }
            AuthMode::ChatgptAuthTokens => {
                if let Some(tokens) = v1.tokens {
                    v2.set_provider_auth(
                        ProviderName::OpenAI,
                        ProviderAuth::ChatgptAuthTokens {
                            tokens,
                            last_refresh: v1.last_refresh,
                        },
                    );
                }
            }
            // Future auth modes added to the protocol enum — ignore for migration
            _ => {}
        }

        v2
    }
}

/// Deserialize auth from JSON, auto-detecting v1 vs v2 format.
pub(super) fn deserialize_auth(json: &str) -> Result<AuthDotJsonV2, serde_json::Error> {
    // Try v2 first (has "version" field)
    if let Ok(v2) = serde_json::from_str::<AuthDotJsonV2>(json)
        && v2.version == 2
    {
        return Ok(v2);
    }
    // Fall back to v1 and convert
    let v1: AuthDotJson = serde_json::from_str(json)?;
    Ok(AuthDotJsonV2::from(v1))
}

/// Expected structure for $ORBIT_HOME/auth.json (v1 legacy format).
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct AuthDotJson {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_mode: Option<AuthMode>,

    #[serde(rename = "OPENAI_API_KEY")]
    pub openai_api_key: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens: Option<TokenData>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_refresh: Option<DateTime<Utc>>,
}

pub(super) fn get_auth_file(orbit_code_home: &Path) -> PathBuf {
    orbit_code_home.join("auth.json")
}

pub(super) fn delete_file_if_exists(orbit_code_home: &Path) -> std::io::Result<bool> {
    let auth_file = get_auth_file(orbit_code_home);
    match std::fs::remove_file(&auth_file) {
        Ok(()) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err),
    }
}

pub(crate) trait AuthStorageBackend: Debug + Send + Sync {
    fn load(&self) -> std::io::Result<Option<AuthDotJsonV2>>;
    fn save(&self, auth: &AuthDotJsonV2) -> std::io::Result<()>;
    fn delete(&self) -> std::io::Result<bool>;
    /// Delete auth for a single provider, preserving others.
    fn delete_provider(&self, provider: ProviderName) -> std::io::Result<bool> {
        if let Some(mut v2) = self.load()? {
            let removed_active = v2.remove_provider_auth(provider).is_some();
            let removed_alt = v2.alternate_credentials.remove(&provider).is_some();
            v2.preferred_auth_modes.remove(&provider);
            let removed = removed_active || removed_alt;
            if removed {
                if v2.has_any_auth() {
                    self.save(&v2)?;
                } else {
                    self.delete()?;
                }
            }
            Ok(removed)
        } else {
            Ok(false)
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct FileAuthStorage {
    orbit_code_home: PathBuf,
}

impl FileAuthStorage {
    pub(super) fn new(orbit_code_home: PathBuf) -> Self {
        Self { orbit_code_home }
    }

    /// Read raw JSON from the auth file.
    fn read_raw(&self) -> std::io::Result<Option<String>> {
        let auth_file = get_auth_file(&self.orbit_code_home);
        match File::open(&auth_file) {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                Ok(Some(contents))
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Create a backup of v1 auth file before first v2 write.
    fn backup_v1_if_needed(&self) -> std::io::Result<()> {
        let auth_file = get_auth_file(&self.orbit_code_home);
        let backup_file = self.orbit_code_home.join("auth.v1.json.bak");
        if auth_file.exists() && !backup_file.exists() {
            // Check if current file is v1 format (no "version" field)
            if let Ok(contents) = std::fs::read_to_string(&auth_file)
                && !contents.contains("\"version\"")
            {
                std::fs::copy(&auth_file, &backup_file)?;
                warn!(
                    "Migrated auth.json from v1 to v2 format. \
                         Backup saved to auth.v1.json.bak. \
                         Older Orbit Code versions cannot read v2 format."
                );
            }
        }
        Ok(())
    }

    fn write_json(&self, json_data: &str) -> std::io::Result<()> {
        let auth_file = get_auth_file(&self.orbit_code_home);
        if let Some(parent) = auth_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut options = OpenOptions::new();
        options.truncate(true).write(true).create(true);
        #[cfg(unix)]
        {
            options.mode(0o600);
        }
        let mut file = options.open(auth_file)?;
        file.write_all(json_data.as_bytes())?;
        file.flush()?;
        Ok(())
    }
}

impl AuthStorageBackend for FileAuthStorage {
    fn load(&self) -> std::io::Result<Option<AuthDotJsonV2>> {
        match self.read_raw()? {
            Some(contents) => {
                let v2 = deserialize_auth(&contents)?;
                Ok(Some(v2))
            }
            None => Ok(None),
        }
    }

    fn save(&self, auth: &AuthDotJsonV2) -> std::io::Result<()> {
        self.backup_v1_if_needed()?;
        let json_data = serde_json::to_string_pretty(auth)?;
        self.write_json(&json_data)
    }

    fn delete(&self) -> std::io::Result<bool> {
        delete_file_if_exists(&self.orbit_code_home)
    }
}

const KEYRING_SERVICE: &str = "Orbit Code Auth";

// Turns the Orbit Code home path into a stable, short key string.
fn compute_store_key(orbit_code_home: &Path) -> std::io::Result<String> {
    let canonical = orbit_code_home
        .canonicalize()
        .unwrap_or_else(|_| orbit_code_home.to_path_buf());
    let path_str = canonical.to_string_lossy();
    let mut hasher = Sha256::new();
    hasher.update(path_str.as_bytes());
    let digest = hasher.finalize();
    let hex = format!("{digest:x}");
    let truncated = hex.get(..16).unwrap_or(&hex);
    Ok(format!("cli|{truncated}"))
}

#[derive(Clone, Debug)]
struct KeyringAuthStorage {
    orbit_code_home: PathBuf,
    keyring_store: Arc<dyn KeyringStore>,
}

impl KeyringAuthStorage {
    fn new(orbit_code_home: PathBuf, keyring_store: Arc<dyn KeyringStore>) -> Self {
        Self {
            orbit_code_home,
            keyring_store,
        }
    }

    fn load_from_keyring(
        &self,
        service: &str,
        key: &str,
    ) -> std::io::Result<Option<AuthDotJsonV2>> {
        match self.keyring_store.load(service, key) {
            Ok(Some(serialized)) => deserialize_auth(&serialized).map(Some).map_err(|err| {
                std::io::Error::other(format!(
                    "failed to deserialize CLI auth from keyring: {err}"
                ))
            }),
            Ok(None) => Ok(None),
            Err(error) => Err(std::io::Error::other(format!(
                "failed to load CLI auth from keyring: {}",
                error.message()
            ))),
        }
    }

    fn save_to_keyring(&self, service: &str, key: &str, value: &str) -> std::io::Result<()> {
        match self.keyring_store.save(service, key, value) {
            Ok(()) => Ok(()),
            Err(error) => {
                let message = format!(
                    "failed to write OAuth tokens to keyring: {}",
                    error.message()
                );
                warn!("{message}");
                Err(std::io::Error::other(message))
            }
        }
    }
}

impl AuthStorageBackend for KeyringAuthStorage {
    fn load(&self) -> std::io::Result<Option<AuthDotJsonV2>> {
        let key = compute_store_key(&self.orbit_code_home)?;
        self.load_from_keyring(KEYRING_SERVICE, &key)
    }

    fn save(&self, auth: &AuthDotJsonV2) -> std::io::Result<()> {
        let key = compute_store_key(&self.orbit_code_home)?;
        let serialized = serde_json::to_string(auth).map_err(std::io::Error::other)?;
        self.save_to_keyring(KEYRING_SERVICE, &key, &serialized)?;
        if let Err(err) = delete_file_if_exists(&self.orbit_code_home) {
            warn!("failed to remove CLI auth fallback file: {err}");
        }
        Ok(())
    }

    fn delete(&self) -> std::io::Result<bool> {
        let key = compute_store_key(&self.orbit_code_home)?;
        let keyring_removed = match self.keyring_store.delete(KEYRING_SERVICE, &key) {
            Ok(removed) => removed,
            Err(err) => {
                warn!("failed to delete auth from keyring service {KEYRING_SERVICE}: {err}");
                false
            }
        };
        let file_removed = delete_file_if_exists(&self.orbit_code_home)?;
        Ok(keyring_removed || file_removed)
    }
}

#[derive(Clone, Debug)]
struct AutoAuthStorage {
    keyring_storage: Arc<KeyringAuthStorage>,
    file_storage: Arc<FileAuthStorage>,
}

impl AutoAuthStorage {
    fn new(orbit_code_home: PathBuf, keyring_store: Arc<dyn KeyringStore>) -> Self {
        Self {
            keyring_storage: Arc::new(KeyringAuthStorage::new(
                orbit_code_home.clone(),
                keyring_store,
            )),
            file_storage: Arc::new(FileAuthStorage::new(orbit_code_home)),
        }
    }
}

impl AuthStorageBackend for AutoAuthStorage {
    fn load(&self) -> std::io::Result<Option<AuthDotJsonV2>> {
        match self.keyring_storage.load() {
            Ok(Some(auth)) => Ok(Some(auth)),
            Ok(None) => self.file_storage.load(),
            Err(err) => {
                warn!("failed to load CLI auth from keyring, falling back to file storage: {err}");
                self.file_storage.load()
            }
        }
    }

    fn save(&self, auth: &AuthDotJsonV2) -> std::io::Result<()> {
        match self.keyring_storage.save(auth) {
            Ok(()) => Ok(()),
            Err(err) => {
                warn!("failed to save auth to keyring, falling back to file storage: {err}");
                self.file_storage.save(auth)
            }
        }
    }

    fn delete(&self) -> std::io::Result<bool> {
        // Keyring storage will delete from disk as well
        self.keyring_storage.delete()
    }
}

// A global in-memory store for mapping orbit_code_home -> AuthDotJsonV2.
static EPHEMERAL_AUTH_STORE: Lazy<Mutex<HashMap<String, AuthDotJsonV2>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Debug)]
struct EphemeralAuthStorage {
    orbit_code_home: PathBuf,
}

impl EphemeralAuthStorage {
    fn new(orbit_code_home: PathBuf) -> Self {
        Self { orbit_code_home }
    }

    fn with_store<F, T>(&self, action: F) -> std::io::Result<T>
    where
        F: FnOnce(&mut HashMap<String, AuthDotJsonV2>, String) -> std::io::Result<T>,
    {
        let key = compute_store_key(&self.orbit_code_home)?;
        let mut store = EPHEMERAL_AUTH_STORE
            .lock()
            .map_err(|_| std::io::Error::other("failed to lock ephemeral auth storage"))?;
        action(&mut store, key)
    }
}

impl AuthStorageBackend for EphemeralAuthStorage {
    fn load(&self) -> std::io::Result<Option<AuthDotJsonV2>> {
        self.with_store(|store, key| Ok(store.get(&key).cloned()))
    }

    fn save(&self, auth: &AuthDotJsonV2) -> std::io::Result<()> {
        self.with_store(|store, key| {
            store.insert(key, auth.clone());
            Ok(())
        })
    }

    fn delete(&self) -> std::io::Result<bool> {
        self.with_store(|store, key| Ok(store.remove(&key).is_some()))
    }
}

pub(crate) fn create_auth_storage(
    orbit_code_home: PathBuf,
    mode: AuthCredentialsStoreMode,
) -> Arc<dyn AuthStorageBackend> {
    let keyring_store: Arc<dyn KeyringStore> = Arc::new(DefaultKeyringStore);
    create_auth_storage_with_keyring_store(orbit_code_home, mode, keyring_store)
}

fn create_auth_storage_with_keyring_store(
    orbit_code_home: PathBuf,
    mode: AuthCredentialsStoreMode,
    keyring_store: Arc<dyn KeyringStore>,
) -> Arc<dyn AuthStorageBackend> {
    match mode {
        AuthCredentialsStoreMode::File => Arc::new(FileAuthStorage::new(orbit_code_home)),
        AuthCredentialsStoreMode::Keyring => {
            Arc::new(KeyringAuthStorage::new(orbit_code_home, keyring_store))
        }
        AuthCredentialsStoreMode::Auto => {
            Arc::new(AutoAuthStorage::new(orbit_code_home, keyring_store))
        }
        AuthCredentialsStoreMode::Ephemeral => Arc::new(EphemeralAuthStorage::new(orbit_code_home)),
    }
}

#[cfg(test)]
#[path = "storage_tests.rs"]
mod tests;
