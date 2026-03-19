# codex-rs/mcp-server/tests/common/

Shared test utilities for MCP server integration tests. This is a separate helper crate (`mcp_test_support`) used by the test suite.

## What this folder does

Provides reusable infrastructure for spawning the MCP server process, sending/receiving JSON-RPC messages, and creating mock model server responses.

## Key files

| File | Purpose |
|------|---------|
| `lib.rs` | Re-exports all helpers: `McpProcess`, `create_mock_responses_server`, response helpers, `to_response()` generic deserializer, and shell formatting utilities from `core_test_support` |
| `mcp_process.rs` | `McpProcess`: spawns `codex-mcp-server` as a child process, manages stdin/stdout communication for sending JSON-RPC messages and reading responses |
| `mock_model_server.rs` | `create_mock_responses_server()`: sets up a wiremock server that simulates the model API, returning SSE-formatted responses |
| `responses.rs` | Helper functions for creating mock SSE responses: `create_shell_command_sse_response()`, `create_apply_patch_sse_response()`, `create_final_assistant_message_sse_response()` |

## Where it plugs in

- **Consumed by**: `tests/suite/` test modules
- **Depends on**: `rmcp` (JSON-RPC types), `core_test_support`, `wiremock`, `serde_json`
