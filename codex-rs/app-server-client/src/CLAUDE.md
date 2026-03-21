# app-server-client/src

## Purpose

Source code for the `orbit-code-app-server-client` crate. Implements two transport-backed client facades for the app-server JSON-RPC API.

## Key Files

| File | Role |
|------|------|
| `lib.rs` | In-process client implementation. Defines `AppServerClient` that wraps `InProcessClientHandle` with a worker task. Provides `AppServerEvent` enum (unified event type for both transports), `AppServerStartArgs`, `request()`, `typed_request()`, `notify()`, `respond_to_server_request()`, `fail_server_request()`, `next_event()`, `shutdown()`. Also defines `TypedRequestError` for deserialization failures and `RequestResult` type alias. |
| `remote.rs` | WebSocket-backed remote client. `RemoteAppServerClient` connects to a `ws://` app-server endpoint, performs the initialize/initialized handshake, and runs a background worker task that routes JSON-RPC requests/responses, server requests, and notifications through bounded channels. Provides the same `request()`, `typed_request()`, `notify()`, `respond_to_server_request()`, `next_event()`, `shutdown()` API surface as the in-process client. |

## What It Plugs Into

- `lib.rs` depends on `orbit-code-app-server::in_process` for the embedded runtime.
- `remote.rs` depends on `tokio-tungstenite` for WebSocket connectivity.
- Both paths produce `AppServerEvent` values consumed by TUI/exec surfaces.

## Imports From

- `orbit-code-app-server::in_process` -- `InProcessStartArgs`, `InProcessClientHandle`, `InProcessServerEvent`.
- `orbit-code-app-server-protocol` -- All typed request/response/notification enums, JSON-RPC primitives.
- `orbit-code-core` -- Config, auth manager, thread manager.
- `orbit-code-protocol` -- `SessionSource`.

## Exports To

- Public API consumed by `orbit-code-tui`, exec surface, and other CLI callers.
