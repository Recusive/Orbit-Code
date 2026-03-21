# app-server/src

## Module Categories

**Entry points:** `lib.rs` (server bootstrap, two-task event loop, config loading, tracing setup), `main.rs` (binary entry point, `--listen` CLI arg parsing).

**Transport:** `transport.rs` (stdio + WebSocket transport implementations, `AppServerTransport` enum, `TransportEvent`), `in_process.rs` (in-memory channel transport for embedding without process boundaries), `outgoing_message.rs` (`OutgoingMessage` enum, `OutgoingMessageSender`, `ConnectionId`).

**Request processing:** `message_processor.rs` (top-level router: initialize handshake, delegates to domain handlers), `orbit_code_message_processor.rs` (core agent-domain handler: threads, turns, models, plugins, auth, MCP, reviews, analytics). Note: `orbit_code_message_processor.rs` coexists with `orbit_code_message_processor/` directory containing extracted helpers (`apps_list_helpers.rs`, `plugin_app_helpers.rs`).

**Domain handlers:** `config_api.rs` (config CRUD), `fs_api.rs` (filesystem operations), `external_agent_config_api.rs` (external agent config detection/import), `command_exec.rs` (PTY-based command execution), `fuzzy_file_search.rs` (file search sessions), `dynamic_tools.rs` (dynamic tool calls), `models.rs` (model listing).

**State & routing:** `thread_state.rs` (per-thread runtime state), `thread_status.rs` (thread status change notifications), `filters.rs` (notification filtering/routing), `bespoke_event_handling.rs` (custom event transformation -- very large file).

**Supporting:** `error_code.rs` (JSON-RPC error constants), `server_request_error.rs` (error mapping), `app_server_tracing.rs` (request tracing spans).

**Binaries:** `bin/notify_capture.rs`, `bin/test_notify_capture.rs`.

**Message processor submodules:** `message_processor/tracing_tests.rs`.
