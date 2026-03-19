# codex-rs/exec-server/src/

This file applies to `codex-rs/exec-server/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-exec-server` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-exec-server`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source code for the `codex-exec-server` crate.

### What this folder does

Implements both the server and client sides of a JSON-RPC 2.0 protocol over WebSocket for remote command execution.

### Key files and their roles

- `lib.rs` -- Public API re-exports: client types, protocol types, and server entry points.
- `protocol.rs` -- Wire protocol definitions: `InitializeParams` (client_name), `InitializeResponse`, and method name constants (`initialize`, `initialized`).
- `client.rs` -- `ExecServerClient`: supports two backends (`Remote` via WebSocket RPC, `InProcess` via `LocalBackend`). Handles initialize/initialized handshake with timeouts.
- `client_api.rs` -- Configuration structs: `ExecServerClientConnectOptions` (client_name, initialize_timeout), `RemoteExecServerConnectArgs` (adds websocket_url, connect_timeout).
- `connection.rs` -- `JsonRpcConnection`: transport abstraction that handles reading/writing JSON-RPC messages over WebSocket streams (and stdio for testing). Splits into reader/writer async tasks.
- `rpc.rs` -- `RpcClient`: JSON-RPC client that manages pending requests by ID, supports `call()` (request/response) and `notify()` (fire-and-forget). Handles out-of-order response matching.
- `server.rs` -- Server module entry point; re-exports `ExecServerHandler`, `DEFAULT_LISTEN_URL`, and `run_main`/`run_main_with_listen_url`.

### Subfolders

- `bin/` -- Binary entry point
- `client/` -- Client-side local backend
- `server/` -- Server-side handler, JSON-RPC processing, and transport
