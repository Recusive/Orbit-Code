# codex-rs/mcp-server/src/

Source directory for the `codex-mcp-server` crate.

## What this folder does

Implements the MCP server that exposes Codex as tools over the JSON-RPC stdio protocol. The architecture uses three concurrent tokio tasks: a stdin reader, a message processor, and a stdout writer, connected via bounded/unbounded channels.

## Key files

| File | Purpose |
|------|---------|
| `lib.rs` | `run_main()`: initializes config, sets up OpenTelemetry, creates the three-task pipeline (stdin reader, message processor, stdout writer) connected by `mpsc` channels |
| `main.rs` | Binary entry point; delegates to `run_main()` via `arg0_dispatch_or_else` |
| `codex_tool_config.rs` | Defines `CodexToolCallParam` (initial session params: prompt, model, profile, cwd, approval policy, sandbox mode, config overrides, instructions) and `CodexToolCallReplyParam` (continue session: thread ID + prompt). Generates JSON Schema via `schemars` for MCP tool definitions. `into_config()` converts params into a `codex-core` `Config` |
| `codex_tool_runner.rs` | `run_codex_tool_session()` and `run_codex_tool_session_reply()`: start/continue a Codex thread, stream events as notifications, handle turn completion, errors, exec approval, and patch approval. `create_call_tool_result_with_thread_id()` formats MCP responses with structured content |
| `message_processor.rs` | `MessageProcessor`: handles `initialize` (returns capabilities + tool list), `tools/list`, `tools/call` (dispatches `codex` and `codex-reply`), `cancelled` notifications, and elicitation responses. Manages a `ThreadManager` for session lifecycle |
| `outgoing_message.rs` | `OutgoingMessageSender`: sends JSON-RPC responses, requests (with oneshot callback), and notifications. Converts Codex `Event` objects into MCP custom notifications with metadata (request ID, thread ID) |
| `exec_approval.rs` | `ExecApprovalElicitRequestParams` and `handle_exec_approval_request()`: sends an MCP elicitation request for command execution approval, waits for the client's `ExecApprovalResponse`, and forwards the decision to the Codex thread |
| `patch_approval.rs` | `PatchApprovalElicitRequestParams` and `handle_patch_approval_request()`: sends an MCP elicitation request for file patch approval with change details |
| `tool_handlers/` | Modular tool call handler implementations |

## Imports / exports

- **Imports from workspace**: `codex-core` (`ThreadManager`, `CodexThread`, `Config`, auth), `codex-protocol` (events, ops, thread IDs), `codex-arg0`, `codex-shell-command`, `codex-utils-cli`, `codex-utils-json-to-toml`
- **External deps**: `rmcp` (MCP types), `schemars` (JSON schema), `serde`, `serde_json`, `tokio`, `tracing`
- **Exports**: `run_main()`, `CodexToolCallParam`, `CodexToolCallReplyParam`, approval types
