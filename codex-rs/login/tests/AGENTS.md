# codex-rs/login/tests/

This file applies to `codex-rs/login/tests/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-login` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-login`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Integration tests for the `codex-login` crate.

### What this folder does

End-to-end tests for both the browser OAuth flow and the device code flow. Uses mock HTTP servers (wiremock, tiny_http) to simulate the auth issuer and validates credential persistence.

### Structure

- `all.rs` -- Single integration test binary entry point; imports the `suite` module.
- `suite/` -- Contains the actual test modules.

### Dependencies

- `wiremock` -- Mock HTTP server for device code flow tests
- `tiny_http` -- Lightweight mock issuer for browser flow tests
- `codex-core` (auth loading), `codex-login`, `core_test_support` (network skip macro)
- `tempfile`, `reqwest`, `anyhow`, `base64`, `serde_json`
