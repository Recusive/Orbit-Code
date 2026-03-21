# app-server-client

## Purpose

The `orbit-code-app-server-client` crate provides a high-level client facade for interacting with the app-server. It supports two transport modes:

1. **In-process** -- Wraps `orbit_code_app_server::in_process` with a worker task, providing async request/response helpers, surface-specific startup policy, and bounded shutdown. Used by the TUI and exec surfaces.
2. **Remote (WebSocket)** -- `RemoteAppServerClient` connects to a running app-server instance over WebSocket, performing the initialize/initialized handshake and providing the same event-driven API.

Both transports expose a unified `AppServerEvent` enum so callers can switch between in-process and remote without changing their higher-level session logic.

## What It Plugs Into

- **Consumed by:** TUI (`orbit-code-tui`), exec surface, and any CLI surface that needs to talk to the app-server.
- **Wraps:** `orbit-code-app-server::in_process` for the in-process transport path.
- **Connects to:** A running `orbit-code-app-server` WebSocket endpoint for the remote transport path.

## Key Exports

- `AppServerClient` -- Main in-process client struct with `request()`, `typed_request()`, `notify()`, `respond_to_server_request()`, `fail_server_request()`, `next_event()`, `shutdown()`.
- `AppServerEvent` -- Unified event enum: `ServerNotification`, `LegacyNotification`, `ServerRequest`, `Lagged`, `Disconnected`.
- `RemoteAppServerClient` / `RemoteAppServerConnectArgs` -- WebSocket-backed remote client.
- `RequestResult` -- Type alias for JSON-RPC result/error pairs.
- `DEFAULT_IN_PROCESS_CHANNEL_CAPACITY`, `InProcessServerEvent` -- Re-exports from the in-process module.

## Key Files

| File | Role |
|------|------|
| `Cargo.toml` | Crate manifest |
| `src/lib.rs` | In-process client facade, `AppServerEvent`, startup logic, worker task |
| `src/remote.rs` | WebSocket-backed remote client with initialize handshake, JSON-RPC routing, and event streaming |

## Imports From

- `orbit-code-app-server` -- `in_process` module for the embedded runtime.
- `orbit-code-app-server-protocol` -- All JSON-RPC and typed protocol types.
- `orbit-code-core` -- `Config`, `AuthManager`, `ThreadManager`, config loader.
- `orbit-code-protocol` -- `SessionSource`.
- `orbit-code-feedback` -- Feedback sink.
- `tokio-tungstenite` -- WebSocket client for remote transport.

## Exports To

- Used by TUI, exec, and any other CLI surface that needs app-server communication.
