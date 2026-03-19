# codex-rs/exec-server/

This file applies to `codex-rs/exec-server/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-exec-server` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-exec-server`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

JSON-RPC server and client for remote command execution. Provides a WebSocket-based server (`codex-exec-server` binary) and a client library for connecting to it either remotely or in-process.

### What this folder does

Implements a lightweight JSON-RPC 2.0 server over WebSocket that accepts connections from Codex clients. The server handles an `initialize`/`initialized` handshake protocol. The client library supports both remote WebSocket connections and an in-process local backend (bypassing network entirely).

### What it plugs into

- **codex-app-server-protocol** -- JSON-RPC message types (JSONRPCMessage, JSONRPCRequest, JSONRPCResponse, JSONRPCError, JSONRPCNotification, RequestId)
- **tokio-tungstenite** -- WebSocket transport layer
- Used by `codex-core` as a remote execution backend

### Imports from

- `codex-app-server-protocol`: JSONRPCMessage, JSONRPCRequest, JSONRPCResponse, JSONRPCError, JSONRPCNotification, JSONRPCErrorError, RequestId
- `tokio-tungstenite`: WebSocket client/server
- `futures`: Stream/Sink extensions for WebSocket
- `clap`: CLI argument parsing for the binary

### Exports to

- `ExecServerClient` -- client for connecting to the server (WebSocket or in-process)
- `ExecServerError` -- error type for client operations
- `ExecServerClientConnectOptions`, `RemoteExecServerConnectArgs` -- connection configuration
- `InitializeParams`, `InitializeResponse` -- handshake protocol types
- `run_main()`, `run_main_with_listen_url()` -- server entry points
- `DEFAULT_LISTEN_URL` -- default bind address (`ws://127.0.0.1:0`)
- `ExecServerListenUrlParseError` -- URL parsing error

### Key files

- `Cargo.toml` -- crate metadata; binary is `codex-exec-server`, library is `codex-exec-server`
- `README.md` -- usage documentation
- `src/lib.rs` -- public re-exports
- `src/protocol.rs` -- `InitializeParams` and `InitializeResponse` types, method name constants
- `src/client.rs` -- `ExecServerClient` with `connect_in_process()` and `connect_websocket()`, `ExecServerError` enum
- `src/client_api.rs` -- `ExecServerClientConnectOptions` and `RemoteExecServerConnectArgs`
- `src/connection.rs` -- `JsonRpcConnection` abstraction over WebSocket (and stdio for tests)
- `src/rpc.rs` -- `RpcClient` for making JSON-RPC calls, matching responses to requests by ID
- `src/server.rs` -- Server module re-exports
- `src/bin/codex-exec-server.rs` -- Binary entry point with `--listen` flag
