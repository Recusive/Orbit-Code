# codex-rs/exec-server/tests/common/

Shared test infrastructure for `codex-exec-server` integration tests.

## What this folder does

Provides reusable test helpers for spawning the exec-server binary and communicating with it over WebSocket.

## Key files and their roles

- `mod.rs` -- Module entry; imports `exec_server` submodule.
- `exec_server.rs` -- `ExecServerHarness` struct and `exec_server()` factory function. The harness: (1) reserves a random TCP port, (2) spawns `codex-exec-server` binary with `--listen ws://127.0.0.1:PORT`, (3) retries WebSocket connection until the server is ready (up to 5s), (4) provides `send_request()`, `send_notification()`, `send_raw_text()`, `next_event()`, `wait_for_event()`, and `shutdown()` methods. Auto-kills the child process on drop.

## Imports from

- `codex_utils_cargo_bin::cargo_bin` -- locates the built binary
- `codex_app_server_protocol` -- JSON-RPC message types
- `tokio-tungstenite` -- WebSocket client
- `futures` -- Stream/Sink extensions

## Used by

- All integration test files in `codex-rs/exec-server/tests/`
