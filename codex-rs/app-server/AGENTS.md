# codex-rs/app-server/

This file applies to `codex-rs/app-server/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-app-server` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-app-server`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### Purpose

The `codex-app-server` crate is the main application server for the Codex CLI ecosystem. It exposes a JSON-RPC API over stdio or WebSocket transports, accepting client requests (thread management, turn execution, config reads/writes, filesystem operations, fuzzy file search, plugin management, etc.) and forwarding them to the core agent runtime. It is the process boundary between UI clients (TUI, VS Code extension, test harness) and the Codex agent engine.

### What It Plugs Into

- **Downstream (serves):** Any JSON-RPC client connecting via stdio or `ws://` -- the TUI, VS Code extension, `codex-app-server-client` (in-process or remote), and the test client.
- **Upstream (depends on):**
  - `codex-core` -- agent runtime, auth, config, thread management.
  - `codex-app-server-protocol` -- all JSON-RPC message types, request/response/notification enums, and schema generation.
  - `codex-protocol` -- lower-level shared protocol types (ThreadId, SessionSource, events).
  - `codex-feedback`, `codex-state`, `codex-otel` -- telemetry, logging, and observability.
  - `codex-login`, `codex-chatgpt`, `codex-backend-client` -- auth and API integration.
  - `codex-file-search` -- fuzzy file search engine.
  - `codex-rmcp-client` -- MCP server communication.

### Key Exports

- `run_main` / `run_main_with_transport` -- entry points that boot the server event loop.
- `AppServerTransport` -- enum for Stdio vs WebSocket transport selection.
- `in_process` module -- in-memory transport for embedding the server inside a CLI process without a socket boundary.
- `INPUT_TOO_LARGE_ERROR_CODE`, `INVALID_PARAMS_ERROR_CODE` -- public error code constants.

### Architecture

The server runs two concurrent Tokio tasks:

1. **Processor loop** -- receives transport events (connection open/close, incoming JSON-RPC messages), dispatches them through `MessageProcessor` which routes to `CodexMessageProcessor` for thread/turn/agent operations, `ConfigApi` for config CRUD, `FsApi` for file operations, etc.
2. **Outbound router loop** -- takes `OutgoingEnvelope` messages from the processor and writes them to the appropriate per-connection writer, handling broadcast, filtering, and backpressure.

### Key Files

| File | Role |
|------|------|
| `Cargo.toml` | Crate manifest with all workspace dependencies |
| `src/lib.rs` | Server bootstrap, config loading, tracing setup, main event loop |
| `src/main.rs` | Binary entry point, CLI arg parsing (`--listen`) |
| `src/transport.rs` | Stdio and WebSocket transport implementations, connection lifecycle |
| `src/in_process.rs` | In-memory transport for embedding server in-process |
| `src/message_processor.rs` | Top-level request router (initialize, config, FS, delegates to CodexMessageProcessor) |
| `src/codex_message_processor.rs` | Agent-domain request handler (threads, turns, models, plugins, auth, etc.) |
| `src/outgoing_message.rs` | Outgoing message types and sender abstraction |
| `src/thread_state.rs` | Per-thread runtime state |
| `src/thread_status.rs` | Thread status tracking and watch manager |
| `src/config_api.rs` | Config read/write/batch-write handler |
| `src/fs_api.rs` | Filesystem operation handler |
| `src/command_exec.rs` | PTY-based command execution manager |
| `src/fuzzy_file_search.rs` | Fuzzy file search session manager |
| `src/dynamic_tools.rs` | Dynamic tool call handling |
| `src/models.rs` | Model listing and supported-model helpers |
| `src/filters.rs` | Notification filtering logic |
| `src/error_code.rs` | JSON-RPC error code constants |
