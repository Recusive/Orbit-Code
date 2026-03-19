# codex-rs/codex-api/tests/

This file applies to `codex-rs/codex-api/tests/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-api` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-api`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Integration and end-to-end tests for the `codex-api` crate.

### What this folder does

Contains tests that verify API client behavior, model types, SSE stream processing, and WebSocket protocol handling.

### Key files

| File | Role |
|------|------|
| `clients.rs` | Tests for API client construction and configuration |
| `models_integration.rs` | Tests for model type serialization/deserialization |
| `sse_end_to_end.rs` | End-to-end SSE stream processing tests |
| `realtime_websocket_e2e.rs` | End-to-end Realtime WebSocket protocol tests |
