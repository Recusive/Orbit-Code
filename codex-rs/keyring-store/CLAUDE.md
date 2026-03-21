# codex-rs/keyring-store/

OS keyring abstraction for securely storing and retrieving credentials (API keys, encryption keys, OAuth tokens).

## Build & Test
```bash
cargo build -p orbit-code-keyring-store
cargo test -p orbit-code-keyring-store
```

## Architecture

The `KeyringStore` trait provides a `load`/`save`/`delete` interface over the OS credential system. `DefaultKeyringStore` implements it using the `keyring` crate with platform-native backends (macOS Keychain, Linux Secret Service, Windows Credential Manager). A public `MockKeyringStore` in the `tests` module provides an in-memory implementation backed by `HashMap<String, Arc<MockCredential>>` for use in downstream crate tests.

## Key Considerations
- Platform-specific keyring features are set in `Cargo.toml` via `[target.'cfg(...)'.dependencies]` -- these must stay in sync with `rmcp-client/Cargo.toml` which has identical platform gates
- `load()` returns `Ok(None)` for missing entries (not an error); `delete()` returns `Ok(false)` for missing entries
- `MockKeyringStore` is public (not `#[cfg(test)]`) so downstream crates can use it in their tests -- it lives in `pub mod tests`
- `MockKeyringStore::set_error()` allows injecting errors for testing error handling paths
- This crate is single-file (`src/lib.rs`) -- no modules
- All operations include `tracing::trace` logging for debugging credential access
