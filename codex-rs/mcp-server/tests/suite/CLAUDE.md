# codex-rs/mcp-server/tests/suite/

Test module directory aggregated by `tests/all.rs`.

## What this folder does

Contains integration test modules that exercise the MCP server end-to-end by spawning the server process and sending JSON-RPC messages.

## Key files

| File | What it tests |
|------|---------------|
| `mod.rs` | Aggregates `codex_tool` module |
| `codex_tool.rs` | Integration tests for the `codex` MCP tool: verifies tool invocation, session creation, event streaming, tool response format with `threadId` and `content` in structured content, and interaction with mock model servers |

## Test patterns

- Tests spawn the MCP server via `McpProcess` from the `common/` crate
- A wiremock mock server simulates the model API with SSE responses
- JSON-RPC messages are sent via stdin and responses are read from stdout
- Tests validate both the MCP protocol compliance and the Codex-specific structured content format
