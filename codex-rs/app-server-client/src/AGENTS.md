# codex-rs/app-server-client/src/

This file applies to `codex-rs/app-server-client/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-app-server-client` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-app-server-client`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### Purpose

Source code for the `codex-app-server-client` crate. Implements two transport-backed client facades for the app-server JSON-RPC API.

### Key Files

| File | Role |
|------|------|
| `lib.rs` | In-process client implementation. Defines `AppServerClient` that wraps `InProcessClientHandle` with a worker task. Provides `AppServerEvent` enum (unified event type for both transports), `AppServerStartArgs`, `request()`, `typed_request()`, `notify()`, `respond_to_server_request()`, `fail_server_request()`, `next_event()`, `shutdown()`. Also defines `TypedRequestError` for deserialization failures and `RequestResult` type alias. |
| `remote.rs` | WebSocket-backed remote client. `RemoteAppServerClient` connects to a `ws://` app-server endpoint, performs the initialize/initialized handshake, and runs a background worker task that routes JSON-RPC requests/responses, server requests, and notifications through bounded channels. Provides the same `request()`, `typed_request()`, `notify()`, `respond_to_server_request()`, `next_event()`, `shutdown()` API surface as the in-process client. |

### What It Plugs Into

- `lib.rs` depends on `codex-app-server::in_process` for the embedded runtime.
- `remote.rs` depends on `tokio-tungstenite` for WebSocket connectivity.
- Both paths produce `AppServerEvent` values consumed by TUI/exec surfaces.

### Imports From

- `codex-app-server::in_process` -- `InProcessStartArgs`, `InProcessClientHandle`, `InProcessServerEvent`.
- `codex-app-server-protocol` -- All typed request/response/notification enums, JSON-RPC primitives.
- `codex-core` -- Config, auth manager, thread manager.
- `codex-protocol` -- `SessionSource`.

### Exports To

- Public API consumed by `codex-tui`, exec surface, and other CLI callers.
