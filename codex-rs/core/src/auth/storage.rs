use chrono::DateTime;
use chrono::Utc;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use std::collections::HashMap;
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

/// Expected structure for $ORBIT_HOME/auth.json.
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

pub(super) trait AuthStorageBackend: Debug + Send + Sync {
    fn load(&self) -> std::io::Result<Option<AuthDotJson>>;
    fn save(&self, auth: &AuthDotJson) -> std::io::Result<()>;
    fn delete(&self) -> std::io::Result<bool>;
}

#[derive(Clone, Debug)]
pub(super) struct FileAuthStorage {
    orbit_code_home: PathBuf,
}

impl FileAuthStorage {
    pub(super) fn new(orbit_code_home: PathBuf) -> Self {
        Self { orbit_code_home }
    }

    /// Attempt to read and parse the `auth.json` file in the given `ORBIT_HOME` directory.
    /// Returns the full AuthDotJson structure.
    pub(super) fn try_read_auth_json(&self, auth_file: &Path) -> std::io::Result<AuthDotJson> {
        let mut file = File::open(auth_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let auth_dot_json: AuthDotJson = serde_json::from_str(&contents)?;

        Ok(auth_dot_json)
    }
}

impl AuthStorageBackend for FileAuthStorage {
    fn load(&self) -> std::io::Result<Option<AuthDotJson>> {
        let auth_file = get_auth_file(&self.orbit_code_home);
        let auth_dot_json = match self.try_read_auth_json(&auth_file) {
            Ok(auth) => auth,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(err),
        };
        Ok(Some(auth_dot_json))
    }

    fn save(&self, auth_dot_json: &AuthDotJson) -> std::io::Result<()> {
        let auth_file = get_auth_file(&self.orbit_code_home);

        if let Some(parent) = auth_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json_data = serde_json::to_string_pretty(auth_dot_json)?;
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

    fn delete(&self) -> std::io::Result<bool> {
        delete_file_if_exists(&self.orbit_code_home)
    }
}

const KEYRING_SERVICE: &str = "Orbit Code Auth";
const LEGACY_KEYRING_SERVICE: &str = "Codex Auth";

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

fn candidate_store_keys(orbit_code_home: &Path) -> std::io::Result<Vec<String>> {
    let current_key = compute_store_key(orbit_code_home)?;
    let mut keys = vec![current_key.clone()];

    if let Some(file_name) = orbit_code_home.file_name()
        && file_name == ".orbit"
        && let Some(parent) = orbit_code_home.parent()
    {
        let legacy_key = compute_store_key(&parent.join(".codex"))?;
        if legacy_key != current_key {
            keys.push(legacy_key);
        }
    }

    Ok(keys)
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

    fn load_from_keyring(&self, service: &str, key: &str) -> std::io::Result<Option<AuthDotJson>> {
        match self.keyring_store.load(service, key) {
            Ok(Some(serialized)) => serde_json::from_str(&serialized).map(Some).map_err(|err| {
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
    fn load(&self) -> std::io::Result<Option<AuthDotJson>> {
        for key in candidate_store_keys(&self.orbit_code_home)? {
            if let Some(auth) = self.load_from_keyring(KEYRING_SERVICE, &key)? {
                return Ok(Some(auth));
            }
            if let Some(auth) = self.load_from_keyring(LEGACY_KEYRING_SERVICE, &key)? {
                return Ok(Some(auth));
            }
        }

        Ok(None)
    }

    fn save(&self, auth: &AuthDotJson) -> std::io::Result<()> {
        let key = compute_store_key(&self.orbit_code_home)?;
        // Simpler error mapping per style: prefer method reference over closure
        let serialized = serde_json::to_string(auth).map_err(std::io::Error::other)?;
        self.save_to_keyring(KEYRING_SERVICE, &key, &serialized)?;
        if let Err(err) = delete_file_if_exists(&self.orbit_code_home) {
            warn!("failed to remove CLI auth fallback file: {err}");
        }
        Ok(())
    }

    fn delete(&self) -> std::io::Result<bool> {
        let mut keyring_removed = false;
        for key in candidate_store_keys(&self.orbit_code_home)? {
            for service in [KEYRING_SERVICE, LEGACY_KEYRING_SERVICE] {
                match self.keyring_store.delete(service, &key) {
                    Ok(removed) => {
                        keyring_removed |= removed;
                    }
                    Err(err) => {
                        warn!("failed to delete auth from keyring service {service}: {err}");
                    }
                }
            }
        }
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
    fn load(&self) -> std::io::Result<Option<AuthDotJson>> {
        match self.keyring_storage.load() {
            Ok(Some(auth)) => Ok(Some(auth)),
            Ok(None) => self.file_storage.load(),
            Err(err) => {
                warn!("failed to load CLI auth from keyring, falling back to file storage: {err}");
                self.file_storage.load()
            }
        }
    }

    fn save(&self, auth: &AuthDotJson) -> std::io::Result<()> {
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

// A global in-memory store for mapping orbit_code_home -> AuthDotJson.
static EPHEMERAL_AUTH_STORE: Lazy<Mutex<HashMap<String, AuthDotJson>>> =
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
        F: FnOnce(&mut HashMap<String, AuthDotJson>, String) -> std::io::Result<T>,
    {
        let key = compute_store_key(&self.orbit_code_home)?;
        let mut store = EPHEMERAL_AUTH_STORE
            .lock()
            .map_err(|_| std::io::Error::other("failed to lock ephemeral auth storage"))?;
        action(&mut store, key)
    }
}

impl AuthStorageBackend for EphemeralAuthStorage {
    fn load(&self) -> std::io::Result<Option<AuthDotJson>> {
        self.with_store(|store, key| Ok(store.get(&key).cloned()))
    }

    fn save(&self, auth: &AuthDotJson) -> std::io::Result<()> {
        self.with_store(|store, key| {
            store.insert(key, auth.clone());
            Ok(())
        })
    }

    fn delete(&self) -> std::io::Result<bool> {
        self.with_store(|store, key| Ok(store.remove(&key).is_some()))
    }
}

pub(super) fn create_auth_storage(
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
