# codex-rs/rmcp-client/

MCP (Model Context Protocol) client library built on the official `rmcp` Rust SDK. Supports stdio and Streamable HTTP transports with OAuth authentication.

## Build & Test
```bash
cargo build -p orbit-code-rmcp-client
cargo test -p orbit-code-rmcp-client
```

## Architecture

`RmcpClient` manages the lifecycle of MCP connections, supporting both stdio (child-process) and Streamable HTTP transports. For HTTP, it handles OAuth authentication via the full authorization-code flow (local callback server + browser launch), token persistence (OS keyring with fallback to `ORBIT_HOME/.credentials.json`), and automatic session recovery on 404 expiry. The client wraps the `rmcp` SDK's service abstraction, performing the MCP initialize handshake and providing methods for tool calls, resource reads, and custom requests. Elicitation requests from MCP servers are forwarded via a callback trait.

## Key Considerations
- `rmcp` has many feature flags enabled (auth, client, server, transport-child-process, transport-streamable-http-client-reqwest, transport-streamable-http-server) -- adding/removing features affects the API surface significantly
- OAuth tokens are stored in the OS keyring via `orbit-code-keyring-store`, with a JSON file fallback in `ORBIT_HOME/.credentials.json`
- Auth status discovery uses RFC 8414 well-known paths to probe whether an MCP server supports OAuth
- Platform-specific keyring features in `Cargo.toml` (apple-native, linux-native-async-persistent, windows-native, sync-secret-service) -- must stay in sync with `keyring-store` crate
- `create_env_for_mcp_server()` in `utils.rs` builds a sanitized environment from an allowlist -- MCP child processes do NOT inherit the full environment
- Integration tests in `tests/` use binaries from `src/bin/` (test stdio and streamable HTTP servers)
- Process-group cleanup for stdio MCP servers uses `orbit-code-utils-pty`
