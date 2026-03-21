# app-server

JSON-RPC application server bridging UI clients (TUI, VS Code extension, test harness) to the core agent runtime.

## Build & Test

```bash
cargo test -p orbit-code-app-server        # Run tests
just fix -p orbit-code-app-server          # Clippy
just fmt                                    # Format
```

Tests live in `tests/all.rs` -> `tests/suite/`. Shared test utilities are in `tests/common/`.

## Architecture

### Two-Task Event Loop

The server runs two concurrent Tokio tasks (bootstrapped in `lib.rs`):

1. **Processor loop** -- receives `TransportEvent`s (connection open/close, incoming JSON-RPC messages) and dispatches them through `MessageProcessor`.
2. **Outbound router loop** -- takes `OutgoingEnvelope` messages from the processor and writes them to the appropriate per-connection writer, handling broadcast and filtering.

### Request Processing Pipeline

```
Transport (stdio/ws/in-process)
  -> TransportEvent
    -> MessageProcessor (message_processor.rs)
      -> Initialize handshake
      -> ConfigApi (config read/write)
      -> FsApi (filesystem ops)
      -> ExternalAgentConfigApi
      -> CodexMessageProcessor (orbit_code_message_processor.rs)
        -> Thread/turn management
        -> Model listing
        -> Plugin/MCP operations
        -> Auth, analytics, reviews
```

`MessageProcessor` handles the initialize handshake and routes to domain-specific handlers. Most agent-domain logic (threads, turns, models, plugins, auth) is delegated to `CodexMessageProcessor`.

### Transport Layer

Three transport modes, all producing the same `TransportEvent` stream:

- **Stdio** -- line-delimited JSON over stdin/stdout
- **WebSocket** -- axum-based WS acceptor with `/health` and `/readyz` endpoints
- **In-process** -- channel-based in-memory transport (`in_process.rs`) for embedding the server inside a CLI process without a socket boundary

### The `orbit_code_message_processor` Dual Structure

`orbit_code_message_processor.rs` is a single large file (~338K) that also has a companion directory `orbit_code_message_processor/` containing helper submodules (`apps_list_helpers.rs`, `plugin_app_helpers.rs`). This is the Rust module + directory coexistence pattern -- the `.rs` file is the module root and the directory holds extracted helpers.

## Key Considerations

- **`orbit_code_message_processor.rs` is extremely large.** When adding new request handlers, extract helper logic into the `orbit_code_message_processor/` subdirectory rather than growing the main file further.
- **`bespoke_event_handling.rs` is also very large** (~146K). It handles custom event transformation for specific notification types.
- **Connection lifecycle matters.** The outbound router tracks per-connection state. If you add new notification types, ensure they flow through the `filters.rs` logic correctly.
- **In-process transport** is used by the TUI to embed the app-server without process boundaries. Changes to the transport abstraction must keep the in-process path working.
- **Mirror TUI changes.** If the app-server's message handling affects what the TUI displays, ensure `tui_app_server/` stays in sync per the root CLAUDE.md convention.
- **Error codes.** Use constants from `error_code.rs` (`INVALID_PARAMS_ERROR_CODE`, `INPUT_TOO_LARGE_ERROR_CODE`, etc.) rather than raw integers.
