# codex-rs/app-server-client/

This file applies to `codex-rs/app-server-client/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-app-server-client` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-app-server-client`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### Purpose

The `codex-app-server-client` crate provides a high-level client facade for interacting with the app-server. It supports two transport modes:

1. **In-process** -- Wraps `codex_app_server::in_process` with a worker task, providing async request/response helpers, surface-specific startup policy, and bounded shutdown. Used by the TUI and exec surfaces.
2. **Remote (WebSocket)** -- `RemoteAppServerClient` connects to a running app-server instance over WebSocket, performing the initialize/initialized handshake and providing the same event-driven API.

Both transports expose a unified `AppServerEvent` enum so callers can switch between in-process and remote without changing their higher-level session logic.

### What It Plugs Into

- **Consumed by:** TUI (`codex-tui`), exec surface, and any CLI surface that needs to talk to the app-server.
- **Wraps:** `codex-app-server::in_process` for the in-process transport path.
- **Connects to:** A running `codex-app-server` WebSocket endpoint for the remote transport path.

### Key Exports

- `AppServerClient` -- Main in-process client struct with `request()`, `typed_request()`, `notify()`, `respond_to_server_request()`, `fail_server_request()`, `next_event()`, `shutdown()`.
- `AppServerEvent` -- Unified event enum: `ServerNotification`, `LegacyNotification`, `ServerRequest`, `Lagged`, `Disconnected`.
- `RemoteAppServerClient` / `RemoteAppServerConnectArgs` -- WebSocket-backed remote client.
- `RequestResult` -- Type alias for JSON-RPC result/error pairs.
- `DEFAULT_IN_PROCESS_CHANNEL_CAPACITY`, `InProcessServerEvent` -- Re-exports from the in-process module.

### Key Files

| File | Role |
|------|------|
| `Cargo.toml` | Crate manifest |
| `src/lib.rs` | In-process client facade, `AppServerEvent`, startup logic, worker task |
| `src/remote.rs` | WebSocket-backed remote client with initialize handshake, JSON-RPC routing, and event streaming |

### Imports From

- `codex-app-server` -- `in_process` module for the embedded runtime.
- `codex-app-server-protocol` -- All JSON-RPC and typed protocol types.
- `codex-core` -- `Config`, `AuthManager`, `ThreadManager`, config loader.
- `codex-protocol` -- `SessionSource`.
- `codex-feedback` -- Feedback sink.
- `tokio-tungstenite` -- WebSocket client for remote transport.

### Exports To

- Used by TUI, exec, and any other CLI surface that needs app-server communication.
