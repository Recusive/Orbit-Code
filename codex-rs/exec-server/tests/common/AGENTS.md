# codex-rs/exec-server/tests/common/

This file applies to `codex-rs/exec-server/tests/common/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-exec-server` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-exec-server`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Shared test infrastructure for `codex-exec-server` integration tests.

### What this folder does

Provides reusable test helpers for spawning the exec-server binary and communicating with it over WebSocket.

### Key files and their roles

- `mod.rs` -- Module entry; imports `exec_server` submodule.
- `exec_server.rs` -- `ExecServerHarness` struct and `exec_server()` factory function. The harness: (1) reserves a random TCP port, (2) spawns `codex-exec-server` binary with `--listen ws://127.0.0.1:PORT`, (3) retries WebSocket connection until the server is ready (up to 5s), (4) provides `send_request()`, `send_notification()`, `send_raw_text()`, `next_event()`, `wait_for_event()`, and `shutdown()` methods. Auto-kills the child process on drop.

### Imports from

- `codex_utils_cargo_bin::cargo_bin` -- locates the built binary
- `codex_app_server_protocol` -- JSON-RPC message types
- `tokio-tungstenite` -- WebSocket client
- `futures` -- Stream/Sink extensions

### Used by

- All integration test files in `codex-rs/exec-server/tests/`
