# codex-rs/mcp-server/

MCP (Model Context Protocol) server for Codex. Ships as both a library (`codex_mcp_server`) and a standalone binary (`codex-mcp-server`).

## What this folder does

Implements a JSON-RPC stdio-based MCP server that exposes Codex as a tool. External LLM clients (Claude, etc.) can invoke the `codex` and `codex-reply` tools to start and continue Codex coding sessions. The server handles:

- MCP protocol negotiation (`initialize` / `initialized`)
- Tool listing (`tools/list`)
- Tool execution (`tools/call` for `codex` and `codex-reply`)
- Streaming events as MCP notifications
- Exec/patch approval via MCP elicitation requests
- Thread management for multi-turn conversations

## Where it plugs in

- **Consumed by**: External MCP clients (e.g., Claude Desktop, other LLM orchestrators) via stdio
- **Depends on**: `codex-core` (session management via `ThreadManager`/`CodexThread`, config, auth), `codex-protocol` (event types, operations), `codex-arg0` (multi-binary dispatch), `codex-shell-command`, `codex-utils-cli`, `codex-utils-json-to-toml`, `rmcp` (MCP protocol types)

## Key files

| File | Role |
|------|------|
| `Cargo.toml` | Crate manifest; binary `codex-mcp-server` + library |
| `src/main.rs` | Binary entry point with arg0 dispatch |
| `src/lib.rs` | Server main loop: stdin reader -> message processor -> stdout writer, with tokio channels |
| `src/codex_tool_config.rs` | `CodexToolCallParam` and `CodexToolCallReplyParam` types with JSON schema generation for MCP tool definitions |
| `src/codex_tool_runner.rs` | Async worker that runs Codex sessions, streams events, handles turn completion/errors |
| `src/message_processor.rs` | Core MCP message handler: dispatches requests/responses/notifications, manages thread lifecycle |
| `src/outgoing_message.rs` | Outgoing message sender with request callback management and notification formatting |
| `src/exec_approval.rs` | Exec approval elicitation: sends MCP elicitation requests for command approval |
| `src/patch_approval.rs` | Patch approval elicitation: sends MCP elicitation requests for file patch approval |
| `src/tool_handlers/` | Individual tool call handlers |
