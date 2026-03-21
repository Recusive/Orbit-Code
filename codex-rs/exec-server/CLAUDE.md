# codex-rs/exec-server/

JSON-RPC server and client for remote command execution. Provides a WebSocket-based server (`orbit-code-exec-server` binary) and a client library for connecting to it either remotely or in-process.

## What this folder does

Implements a lightweight JSON-RPC 2.0 server over WebSocket that accepts connections from Codex clients. The server handles an `initialize`/`initialized` handshake protocol. The client library supports both remote WebSocket connections and an in-process local backend (bypassing network entirely).

## What it plugs into

- **orbit-code-app-server-protocol** -- JSON-RPC message types (JSONRPCMessage, JSONRPCRequest, JSONRPCResponse, JSONRPCError, JSONRPCNotification, RequestId)
- **tokio-tungstenite** -- WebSocket transport layer
- Used by `orbit-code-core` as a remote execution backend

## Imports from

- `orbit-code-app-server-protocol`: JSONRPCMessage, JSONRPCRequest, JSONRPCResponse, JSONRPCError, JSONRPCNotification, JSONRPCErrorError, RequestId
- `tokio-tungstenite`: WebSocket client/server
- `futures`: Stream/Sink extensions for WebSocket
- `clap`: CLI argument parsing for the binary

## Exports to

- `ExecServerClient` -- client for connecting to the server (WebSocket or in-process)
- `ExecServerError` -- error type for client operations
- `ExecServerClientConnectOptions`, `RemoteExecServerConnectArgs` -- connection configuration
- `InitializeParams`, `InitializeResponse` -- handshake protocol types
- `run_main()`, `run_main_with_listen_url()` -- server entry points
- `DEFAULT_LISTEN_URL` -- default bind address (`ws://127.0.0.1:0`)
- `ExecServerListenUrlParseError` -- URL parsing error

## Key files

- `Cargo.toml` -- crate metadata; binary is `orbit-code-exec-server`, library is `orbit-code-exec-server`
- `README.md` -- usage documentation
- `src/lib.rs` -- public re-exports
- `src/protocol.rs` -- `InitializeParams` and `InitializeResponse` types, method name constants
- `src/client.rs` -- `ExecServerClient` with `connect_in_process()` and `connect_websocket()`, `ExecServerError` enum
- `src/client_api.rs` -- `ExecServerClientConnectOptions` and `RemoteExecServerConnectArgs`
- `src/connection.rs` -- `JsonRpcConnection` abstraction over WebSocket (and stdio for tests)
- `src/rpc.rs` -- `RpcClient` for making JSON-RPC calls, matching responses to requests by ID
- `src/server.rs` -- Server module re-exports
- `src/bin/codex-exec-server.rs` -- Binary entry point with `--listen` flag
