# codex-rs/utils/rustls-provider/

This file applies to `codex-rs/utils/rustls-provider/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-utils-rustls-provider` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-utils-rustls-provider`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Crate `codex-utils-rustls-provider` -- one-time rustls crypto provider initialization.

### What this folder does

Ensures exactly one process-wide rustls crypto provider is installed. This is necessary because rustls cannot auto-select a provider when both `ring` and `aws-lc-rs` features are enabled in the dependency graph. Uses `std::sync::Once` for thread-safe initialization.

### Key types and functions

- `ensure_rustls_crypto_provider()` -- installs the `ring` default provider via `Once::call_once`; safe to call multiple times

### Imports from

- `rustls` -- TLS library whose crypto provider is being configured

### Exports to

Called early in startup by `codex-core` and any crate that needs TLS before making HTTP requests.

### Key files

- `Cargo.toml` -- crate metadata; depends on `rustls`
- `src/lib.rs` -- single function `ensure_rustls_crypto_provider` (13 lines)
