# codex-rs/keyring-store/

Crate: `codex-keyring-store` -- OS keyring abstraction for credential storage.

## What this crate does

Provides a trait-based abstraction over the OS credential/keyring system (macOS Keychain, Windows Credential Manager, Linux Secret Service). Used to securely store and retrieve secrets like API keys and encryption keys.

## Main types

- `KeyringStore` trait -- Abstraction for credential storage with three operations:
  - `load(service, account) -> Result<Option<String>>` -- Retrieve a stored credential
  - `save(service, account, value) -> Result<()>` -- Store a credential
  - `delete(service, account) -> Result<bool>` -- Delete a credential (returns whether it existed)
- `DefaultKeyringStore` -- Production implementation using the `keyring` crate with platform-native backends
- `CredentialStoreError` -- Error wrapper around `keyring::Error`
- `MockKeyringStore` (in `tests` module, public) -- In-memory mock for testing, backed by `HashMap<String, Arc<MockCredential>>`

## Platform backends

Configured via platform-specific dependencies in `Cargo.toml`:
- **macOS**: `apple-native` (Keychain)
- **Linux**: `linux-native-async-persistent` (Secret Service / libsecret)
- **Windows**: `windows-native` (Credential Manager)
- **FreeBSD/OpenBSD**: `sync-secret-service`
- All platforms: `crypto-rust` feature for cryptographic operations

## What it plugs into

- Used by `codex-secrets` to store encryption keys for the local secrets backend
- Used by `codex-login` for API key/token storage

## Imports from / exports to

**Dependencies:**
- `keyring` -- Cross-platform credential storage library
- `tracing` -- Debug/trace logging

**Exports:**
- `KeyringStore` trait, `DefaultKeyringStore`, `CredentialStoreError`
- `tests::MockKeyringStore` (public module for test support in downstream crates)

## Key files

- `Cargo.toml` -- Crate manifest with platform-specific keyring feature flags
- `src/lib.rs` -- Single-file implementation with trait, default impl, error type, and mock
