# codex-rs/rmcp-client/

This file applies to `codex-rs/rmcp-client/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-rmcp-client` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-rmcp-client`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

MCP (Model Context Protocol) client library built on top of the official `rmcp` Rust SDK.

### What this folder does

Provides `RmcpClient`, a high-level MCP client that supports both stdio (child-process) and Streamable HTTP transports. Handles OAuth authentication, token persistence, session recovery on 404 expiry, process-group cleanup, and elicitation forwarding.

### What it plugs into

- Used by `codex-core` to connect to user-configured MCP servers (tools, resources, prompts).
- Consumed via the `codex-rmcp-client` crate name.

### Imports from

- `codex-client` -- custom CA certificate support for reqwest.
- `codex-protocol` -- `McpAuthStatus` and protocol types.
- `codex-keyring-store` -- OS keyring abstraction for OAuth credential storage.
- `codex-utils-pty` -- process-group management (terminate/kill on drop).
- `codex-utils-home-dir` -- locating `ORBIT_HOME` for fallback credential files.
- `rmcp` -- the official Rust MCP SDK (client, server, transport, auth).
- `oauth2`, `reqwest`, `serde_json`, `tokio`, `tracing`.

### Exports to

- Re-exports `RmcpClient`, `Elicitation`, `ElicitationResponse`, `SendElicitation`, `ToolWithConnectorId`, `ListToolsWithConnectorIdResult`.
- Re-exports OAuth helpers: `perform_oauth_login`, `save_oauth_tokens`, `delete_oauth_tokens`, `OAuthCredentialsStoreMode`, `StoredOAuthTokens`.
- Re-exports auth-status discovery: `determine_streamable_http_auth_status`, `discover_streamable_http_oauth`, `supports_oauth_login`.

### Key files

- `Cargo.toml` -- crate manifest with `rmcp` feature flags for auth, client, server, streamable-http, and child-process transports.
- `src/lib.rs` -- module declarations and public re-exports.
- `src/rmcp_client.rs` -- `RmcpClient` struct: stdio and HTTP constructors, initialize handshake, tool/resource operations, session recovery, OAuth token refresh and persistence.
- `src/auth_status.rs` -- probes MCP servers for OAuth support via RFC 8414 well-known discovery.
- `src/oauth.rs` -- credential storage/loading across OS keyring and fallback JSON file (`ORBIT_HOME/.credentials.json`).
- `src/perform_oauth_login.rs` -- full OAuth authorization-code flow with local callback server and browser launch.
- `src/program_resolver.rs` -- cross-platform executable resolution (passthrough on Unix, `which` on Windows).
- `src/logging_client_handler.rs` -- `ClientHandler` implementation that logs MCP server notifications (progress, resource updates, tool list changes).
- `src/utils.rs` -- environment-variable allowlisting for child MCP processes, HTTP header construction.
- `src/bin/` -- test server binaries (stdio and streamable HTTP) used by integration tests.
- `tests/` -- integration tests for process-group cleanup, resource listing, and streamable HTTP session recovery.
