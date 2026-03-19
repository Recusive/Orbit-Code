# codex-rs/login/

Authentication and login library for the Codex CLI. Provides both browser-based OAuth and device-code login flows.

## What this folder does

The `codex-login` crate implements two authentication mechanisms:

1. **Browser OAuth flow** -- Starts a local HTTP callback server, opens the browser to the authorization URL, handles the OAuth callback with PKCE, exchanges the code for tokens, and persists credentials.
2. **Device code flow** -- Requests a device code from the server, displays it to the user, polls for authorization, exchanges for tokens, and persists credentials.

Both flows support workspace restrictions, token-exchange for API keys, and configurable credential storage (file-based).

## Where it plugs in

- **Consumed by**: `codex-cli` (login subcommand), `codex-tui` (auth prompts), `codex-app-server` (app server auth initialization)
- **Depends on**: `codex-core` (auth types: `AuthManager`, `CodexAuth`, `AuthDotJson`, `save_auth`, `TokenData`), `codex-client` (custom CA transport), `codex-app-server-protocol` (`AuthMode`), `reqwest`, `tiny_http`, `webbrowser`, `sha2`, `base64`, `rand`

## Main exports

- `run_login_server(opts)` / `LoginServer` -- Browser OAuth flow
- `run_device_code_login(opts)` -- Device code flow
- `request_device_code(opts)` / `complete_device_code_login(opts, code)` -- Split device code API
- `ServerOptions` / `ShutdownHandle` -- Configuration and control types
- Re-exports from `codex-core`: `AuthManager`, `CodexAuth`, `AuthDotJson`, `CLIENT_ID`, `login_with_api_key`, `logout`, `save_auth`, `TokenData`

## Key files

| File | Role |
|------|------|
| `Cargo.toml` | Crate manifest |
| `src/lib.rs` | Public API surface; re-exports from submodules and codex-core |
| `src/server.rs` | Local OAuth callback server: bind with retry, PKCE authorization URL, token exchange, workspace validation, credential persistence, HTML error/success pages |
| `src/device_code_auth.rs` | Device code flow: request user code, poll for token, complete exchange |
| `src/pkce.rs` | PKCE code verifier/challenge generation (S256) |
| `src/assets/` | HTML templates for browser success/error pages |
