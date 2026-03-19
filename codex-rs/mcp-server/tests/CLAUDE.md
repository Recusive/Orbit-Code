# codex-rs/mcp-server/tests/

Integration tests for the `codex-mcp-server` crate.

## What this folder does

End-to-end tests that spawn the MCP server process, send JSON-RPC messages, and verify responses. Uses mock model servers to avoid real API calls.

## Structure

- `all.rs` -- Single integration test binary entry point; imports the `suite` module.
- `common/` -- Shared test utilities (separate crate).
- `suite/` -- Contains the actual test modules.

## Dependencies

- `mcp_test_support` (workspace test support crate)
- `core_test_support` (shell formatting helpers, network skip macro)
- `wiremock` -- Mock HTTP server for model API responses
- `pretty_assertions`, `tempfile`, `os_info`
