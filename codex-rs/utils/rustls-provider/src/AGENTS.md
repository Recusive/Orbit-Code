# codex-rs/utils/rustls-provider/src/

This file applies to `codex-rs/utils/rustls-provider/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-utils-rustls-provider` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-utils-rustls-provider`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source directory for the `codex-utils-rustls-provider` crate.

### Key files

- `lib.rs` -- single-file implementation containing:
  - `ensure_rustls_crypto_provider()` -- uses `std::sync::Once` to call `rustls::crypto::ring::default_provider().install_default()` exactly once per process lifetime
  - The `let _ =` pattern ignores the `Err` returned if another provider was already installed
