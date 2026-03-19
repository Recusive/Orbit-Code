# codex-rs/mcp-server/tests/

This file applies to `codex-rs/mcp-server/tests/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-mcp-server` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-mcp-server`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Integration tests for the `codex-mcp-server` crate.

### What this folder does

End-to-end tests that spawn the MCP server process, send JSON-RPC messages, and verify responses. Uses mock model servers to avoid real API calls.

### Structure

- `all.rs` -- Single integration test binary entry point; imports the `suite` module.
- `common/` -- Shared test utilities (separate crate).
- `suite/` -- Contains the actual test modules.

### Dependencies

- `mcp_test_support` (workspace test support crate)
- `core_test_support` (shell formatting helpers, network skip macro)
- `wiremock` -- Mock HTTP server for model API responses
- `pretty_assertions`, `tempfile`, `os_info`
