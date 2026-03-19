# codex-rs/mcp-server/src/tool_handlers/

Modular tool call handler implementations for the MCP server.

## What this folder does

Contains individual handler modules for MCP tool calls. Each module implements the logic for a specific tool exposed by the Codex MCP server.

## Key files

| File | Purpose |
|------|---------|
| `mod.rs` | Module declarations; re-exports `create_conversation` and `send_message` submodules as `pub(crate)` |
| `create_conversation.rs` | Handler for creating a new Codex conversation/thread |
| `send_message.rs` | Handler for sending a message to an existing conversation |

## Where it plugs in

- **Called by**: `message_processor.rs` when dispatching `tools/call` requests
- **Uses**: `codex-core` thread management, `codex-protocol` operations
