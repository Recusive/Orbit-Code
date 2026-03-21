# codex-rs/mcp-server/src/

MCP server implementation: JSON-RPC message processing, tool execution, and approval elicitation.

## Module Layout

- **message_processor** (`message_processor.rs`) -- Core MCP handler: dispatches `initialize`, `tools/list`, `tools/call`, `cancelled`, and elicitation responses; manages `ThreadManager` for session lifecycle
- **tool_runner** (`orbit_code_tool_runner.rs`) -- Async Codex session execution: starts/continues threads, streams events as notifications, handles turn completion and approval requests
- **tool_config** (`orbit_code_tool_config.rs`) -- `CodexToolCallParam` and `CodexToolCallReplyParam` types with JSON schema generation for MCP tool definitions; `into_config()` for conversion to `orbit-code-core` `Config`
- **outgoing_message** (`outgoing_message.rs`) -- JSON-RPC response/request/notification sender with callback management
- **approval** (`exec_approval.rs`, `patch_approval.rs`) -- MCP elicitation request handling for command execution and file patch approval
- **tool_handlers/** -- Modular per-tool call handler implementations
