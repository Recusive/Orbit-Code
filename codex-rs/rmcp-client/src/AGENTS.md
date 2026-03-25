# codex-rs/rmcp-client/src/

This file applies to `codex-rs/rmcp-client/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-rmcp-client` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-rmcp-client`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source code for the `codex-rmcp-client` MCP client library.

### What this folder does

Contains the implementation of the MCP client, OAuth authentication, auth-status discovery, credential persistence, and utility modules.

### Key files

- `lib.rs` -- module declarations and public API re-exports.
- `rmcp_client.rs` -- core `RmcpClient` implementation: manages client state (Connecting/Ready), creates stdio or streamable-HTTP transports, performs MCP initialize handshake, executes tool calls / resource reads / custom requests, handles session recovery on 404 expiry, and manages OAuth token refresh/persistence.
- `auth_status.rs` -- `determine_streamable_http_auth_status()` probes an MCP server to classify its auth mode (BearerToken, OAuth, NotLoggedIn, Unsupported) using RFC 8414 well-known discovery paths.
- `oauth.rs` -- credential storage layer: loads/saves/deletes OAuth tokens using OS keyring (`keyring` crate) with fallback to `ORBIT_HOME/.credentials.json`. Includes `OAuthPersistor` for automatic persistence after token refresh.
- `perform_oauth_login.rs` -- implements the full OAuth authorization-code flow: spawns a local HTTP callback server, launches the browser, handles the callback, exchanges the code for tokens, and persists them.
- `program_resolver.rs` -- resolves MCP server executable paths. On Unix this is a no-op; on Windows it uses the `which` crate to resolve extensions (.cmd, .bat).
- `logging_client_handler.rs` -- `LoggingClientHandler` implements the rmcp `ClientHandler` trait, forwarding elicitation requests and logging MCP server notifications at appropriate tracing levels.
- `utils.rs` -- helper functions: `create_env_for_mcp_server()` builds a sanitized environment from an allowlist of variables; `build_default_headers()` / `apply_default_headers()` construct HTTP headers from static and env-sourced values.

### Imports from

- `rmcp` SDK for MCP protocol types and service lifecycle.
- `codex-client`, `codex-protocol`, `codex-keyring-store`, `codex-utils-pty`, `codex-utils-home-dir`.

### Exports to

- Parent crate (`codex-rmcp-client`) re-exports the public API defined here.
