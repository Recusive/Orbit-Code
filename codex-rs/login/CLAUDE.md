# codex-rs/login/

Authentication library: browser-based OAuth (PKCE + local callback server) and device-code login flows, with token exchange and credential persistence.

## Build & Test
```bash
cargo build -p orbit-code-login
cargo test -p orbit-code-login
```

## Architecture

The crate implements two authentication flows. The browser OAuth flow (`server.rs`) starts a `tiny_http` server on localhost, generates PKCE codes, opens the browser to the authorization URL, handles the `/auth/callback` with code exchange, validates workspace restrictions, optionally exchanges tokens for API keys, and persists credentials via `orbit-code-core`'s auth module. The device-code flow (`device_code_auth.rs`) requests a user code, displays it for the user, polls for authorization, and uses the same token exchange and persistence path.

Both flows produce `CodexAuth` credentials that are saved to disk. The crate also provides an Anthropic-specific login module (`anthropic.rs`) for Anthropic OAuth. The `pkce.rs` module handles PKCE code generation (S256 challenge).

## Key Considerations

- Re-exports auth types from `orbit-code-core` (`AuthManager`, `CodexAuth`, `AuthDotJson`, `save_auth`, `TokenData`, `CLIENT_ID`) -- consumers should import from this crate, not directly from core.
- The local OAuth server binds to `127.0.0.1` with port retry logic -- it tries a range of ports if the preferred one is taken.
- HTML templates for browser success/error pages are embedded at compile time via `include_str!()` from `src/assets/` -- if you add or modify templates, ensure the `BUILD.bazel` `compile_data` is updated.
- `ServerOptions` contains the full configuration for the OAuth flow including base URL, workspace restrictions, and shutdown signaling via `ShutdownHandle`.
- Uses `orbit-code-client` for custom CA support in token exchange requests (corporate proxy compatibility).
