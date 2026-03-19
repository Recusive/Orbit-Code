# codex-rs/exec-server/src/server/

This file applies to `codex-rs/exec-server/src/server/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-exec-server` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-exec-server`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Server-side implementation for the exec-server.

### What this folder does

Implements the WebSocket server that listens for JSON-RPC connections, processes the initialize/initialized handshake, and dispatches incoming requests to the handler.

### Key files and their roles

- `mod.rs` (server.rs in parent) -- Module entry point. Re-exports `ExecServerHandler`, `DEFAULT_LISTEN_URL`, `ExecServerListenUrlParseError`. Provides `run_main()` and `run_main_with_listen_url()`.
- `handler.rs` -- `ExecServerHandler`: manages connection state (initialize_requested, initialized) with atomic booleans. Enforces single-initialize and ordered handshake.
- `jsonrpc.rs` -- JSON-RPC helper utilities (e.g., `invalid_request()` error constructor).
- `processor.rs` -- `run_connection()`: reads incoming JSON-RPC messages from a `JsonRpcConnection`, routes `initialize` requests and `initialized` notifications to the handler, and sends responses/errors back.
- `transport.rs` -- WebSocket transport: `parse_listen_url()`, `run_transport()`, and `run_websocket_listener()` that binds a `TcpListener`, accepts connections, upgrades to WebSocket, and spawns per-connection tasks.
- `transport_tests.rs` -- Tests for URL parsing and WebSocket transport behavior.

### Imports from

- `crate::connection::JsonRpcConnection`
- `crate::protocol`: InitializeResponse, method constants
- `codex-app-server-protocol`: JSONRPCErrorError
- `tokio`, `tokio-tungstenite`: async networking

### Exports to

- `ExecServerHandler` used by `LocalBackend` (in-process client)
- `DEFAULT_LISTEN_URL` and `run_main*` used by the binary and library consumers
