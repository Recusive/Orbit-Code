# codex-rs/keyring-store/src/

This file applies to `codex-rs/keyring-store/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-keyring-store` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-keyring-store`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source code for the `codex-keyring-store` crate.

### What this folder does

Contains the single-file implementation of the OS keyring abstraction.

### Key files

- `lib.rs` -- Complete crate implementation:
  - **Error type**: `CredentialStoreError` -- Wraps `keyring::Error` with `Display`, `Debug`, and `Error` impls
  - **Trait**: `KeyringStore` -- `Debug + Send + Sync` trait with `load`, `save`, `delete` methods
  - **Default impl**: `DefaultKeyringStore` -- Uses `keyring::Entry` for platform-native credential storage:
    - `load` returns `Ok(None)` for `NoEntry` errors
    - `delete` returns `Ok(false)` for `NoEntry` errors
    - All operations include `tracing::trace` logging
  - **Mock** (public `tests` module): `MockKeyringStore` -- Thread-safe in-memory store for testing:
    - Backed by `Arc<Mutex<HashMap<String, Arc<MockCredential>>>>`
    - `credential(account)` -- Get or create a mock credential
    - `saved_value(account)` -- Read stored value
    - `set_error(account, error)` -- Inject errors for testing
    - `contains(account)` -- Check existence
    - Implements `KeyringStore` trait

### Imports from / exports to

**Imports:**
- `keyring::{Entry, Error, credential::CredentialApi, mock::MockCredential}`
- `tracing::trace`
- `std::sync::{Arc, Mutex}`

**Exports:**
- `CredentialStoreError`, `KeyringStore`, `DefaultKeyringStore`
- `tests::MockKeyringStore` (publicly accessible for downstream test support)
