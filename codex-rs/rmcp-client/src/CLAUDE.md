# codex-rs/rmcp-client/src/

Source for the `orbit-code-rmcp-client` crate -- MCP client with OAuth, session recovery, and multi-transport support.

## Module Layout
- **Client core** (`rmcp_client.rs`): `RmcpClient` managing connection state (Connecting/Ready), transport creation, MCP handshake, tool/resource operations, session recovery, and token refresh
- **Auth discovery** (`auth_status.rs`): `determine_streamable_http_auth_status()` probing MCP servers for OAuth support via RFC 8414 well-known paths
- **OAuth flow** (`perform_oauth_login.rs`, `oauth.rs`): Full authorization-code flow with local callback server and browser launch; credential storage via OS keyring with JSON file fallback
- **Infrastructure** (`logging_client_handler.rs`, `program_resolver.rs`, `utils.rs`): MCP notification logging, cross-platform executable resolution, environment allowlisting for child processes
- **Test servers** (`bin/`): Stdio and streamable HTTP test server binaries for integration tests
