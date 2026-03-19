# codex-rs/exec-server/tests/

Integration tests for the `codex-exec-server` crate.

## What this folder does

Contains integration tests that spawn the `codex-exec-server` binary, connect via WebSocket, and verify the JSON-RPC protocol behavior.

## Key files and their roles

- `initialize.rs` -- Tests the initialize/initialized handshake: spawns the server binary, sends an `initialize` request with `InitializeParams`, verifies the response matches `InitializeResponse {}`, then shuts down.
- `process.rs` -- Tests for WebSocket message processing.
- `websocket.rs` -- Tests for WebSocket transport behavior.
- `common/` -- Shared test infrastructure.

## Imports from

- `codex_exec_server`: `InitializeParams`, `InitializeResponse`
- `codex_app_server_protocol`: `JSONRPCMessage`, `JSONRPCResponse`
- `common::exec_server`: test harness

## Platform notes

- Tests are gated with `#![cfg(unix)]` -- they only run on Unix platforms.
