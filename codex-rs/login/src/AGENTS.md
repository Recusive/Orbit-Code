# codex-rs/login/src/

This file applies to `codex-rs/login/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-login` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-login`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source directory for the `codex-login` crate.

### What this folder does

Implements the Codex CLI authentication flows: a local OAuth callback server for browser-based login and a device-code polling flow for headless environments.

### Key files

| File | Purpose |
|------|---------|
| `lib.rs` | Crate root; declares modules, re-exports public API (`LoginServer`, `ServerOptions`, `ShutdownHandle`, `run_login_server`, `run_device_code_login`, `request_device_code`, `complete_device_code_login`), plus auth types from codex-core |
| `server.rs` | Browser OAuth flow implementation. `run_login_server()` starts a `tiny_http` server on localhost, generates PKCE codes and state, opens the browser, and processes `/auth/callback` (code exchange), `/success` (branded HTML page), and `/cancel` (graceful shutdown). Handles token exchange via `exchange_code_for_tokens()`, workspace validation via `ensure_workspace_allowed()`, API key exchange via `obtain_api_key()`, and credential persistence via `persist_tokens_async()`. Includes URL redaction for secure logging |
| `device_code_auth.rs` | Device code flow. `request_device_code()` calls `/deviceauth/usercode`, returns a `DeviceCode` with verification URL and user code. `complete_device_code_login()` polls `/deviceauth/token` until authorization, then exchanges for tokens via the same code path as the browser flow |
| `pkce.rs` | PKCE generation: 64-byte random verifier (URL-safe base64), SHA-256 challenge. Returns `PkceCodes { code_verifier, code_challenge }` |
| `assets/` | Static HTML templates embedded at compile time via `include_str!()` |

### Imports / exports

- **Imports**: `codex-core` (auth subsystem, token parsing, default client), `codex-client` (custom CA transport builder), `codex-app-server-protocol` (`AuthMode`), `reqwest`, `tiny_http`, `webbrowser`, `base64`, `sha2`, `rand`, `chrono`, `url`, `urlencoding`
- **Exports**: See `lib.rs` re-exports
