# codex-rs/core/src/auth/

This file applies to `codex-rs/core/src/auth/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Credential storage backends for Codex CLI authentication.

### What this folder does

Implements the pluggable storage layer for persisting authentication credentials (API keys, OAuth tokens). Supports four storage modes:

- **File** (`FileAuthStorage`): Reads/writes `auth.json` in `$CODEX_HOME/` with 0600 permissions on Unix.
- **Keyring** (`KeyringAuthStorage`): Stores credentials in the OS keyring (macOS Keychain, Linux Secret Service, Windows Credential Manager). Uses a SHA-256 hash of the codex_home path as the keyring entry key.
- **Auto** (`AutoAuthStorage`): Tries keyring first, falls back to file storage on failure.
- **Ephemeral** (`EphemeralAuthStorage`): In-memory only, using a global `Mutex<HashMap>`. No persistence across process restarts.

### Key files

| File | Purpose |
|------|---------|
| `storage.rs` | `AuthStorageBackend` trait and all four implementations; `AuthDotJson` struct; `create_auth_storage()` factory |
| `storage_tests.rs` | Tests for storage backends |

### Imports from

- `crate::token_data::TokenData` -- OAuth token structure
- `codex_app_server_protocol::AuthMode` -- authentication mode enum
- `codex_keyring_store` -- `KeyringStore` trait and `DefaultKeyringStore` implementation

### Exports to

- `crate::auth` (parent module in `auth.rs`) -- uses `AuthStorageBackend` for the `AuthManager` and `CodexAuth` types
- `crate::config` -- `AuthCredentialsStoreMode` is a config option

### Data format

The `auth.json` file structure:
```json
{
  "auth_mode": "...",
  "OPENAI_API_KEY": "...",
  "tokens": { ... },
  "last_refresh": "2025-01-01T00:00:00Z"
}
```
