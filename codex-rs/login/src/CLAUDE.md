# codex-rs/login/src/

OAuth and device-code authentication flow implementations.

## Module Layout

- **server** (`server.rs`) -- Browser OAuth flow: local `tiny_http` callback server, PKCE authorization URL generation, code exchange, workspace validation, API key exchange, credential persistence, HTML response pages
- **device_code_auth** (`device_code_auth.rs`) -- Device code flow: `request_device_code()` calls `/deviceauth/usercode`, `complete_device_code_login()` polls `/deviceauth/token` until authorized
- **anthropic** (`anthropic.rs`) -- Anthropic-specific OAuth login flow
- **pkce** (`pkce.rs`) -- PKCE code generation: 64-byte random verifier (URL-safe base64), SHA-256 challenge
- **assets/** -- Static HTML templates embedded at compile time for browser success/error pages
- **lib** (`lib.rs`) -- Public API surface and re-exports from submodules and `orbit-code-core` auth types
