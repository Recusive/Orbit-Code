# codex-rs/rmcp-client/src/bin/

Test MCP server binaries used by integration tests and manual testing.

## What this folder does

Contains standalone MCP server binaries that the rmcp-client integration tests spawn as child processes. They are not shipped in production builds.

## Key files

- `rmcp_test_server.rs` -- minimal stdio MCP server with a single `echo` tool. Used by the process-group cleanup test.
- `test_stdio_server.rs` -- richer stdio MCP server with `echo`, `echo-tool`, `image`, and `image_scenario` tools, plus MCP resources (`memo://codex/example-note`) and resource templates. Used by resource listing/reading integration tests and manual TUI image-rendering tests.
- `test_streamable_http_server.rs` -- axum-based streamable HTTP MCP server with `echo` tool, resources, OAuth discovery endpoint, bearer token middleware, and a control endpoint (`/test/control/session-post-failure`) that can inject 404/401/500 failures for session-recovery testing.

## What it plugs into

- `tests/process_group_cleanup.rs` spawns `rmcp_test_server`.
- `tests/resources.rs` spawns `test_stdio_server`.
- `tests/streamable_http_recovery.rs` spawns `test_streamable_http_server`.

## Imports from

- `rmcp` SDK server-side handler traits and model types.
- `axum` (streamable HTTP server only).
- `serde_json`, `tokio`.
