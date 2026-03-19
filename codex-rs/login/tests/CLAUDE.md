# codex-rs/login/tests/

Integration tests for the `codex-login` crate.

## What this folder does

End-to-end tests for both the browser OAuth flow and the device code flow. Uses mock HTTP servers (wiremock, tiny_http) to simulate the auth issuer and validates credential persistence.

## Structure

- `all.rs` -- Single integration test binary entry point; imports the `suite` module.
- `suite/` -- Contains the actual test modules.

## Dependencies

- `wiremock` -- Mock HTTP server for device code flow tests
- `tiny_http` -- Lightweight mock issuer for browser flow tests
- `codex-core` (auth loading), `codex-login`, `core_test_support` (network skip macro)
- `tempfile`, `reqwest`, `anyhow`, `base64`, `serde_json`
