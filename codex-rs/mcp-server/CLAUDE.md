# codex-rs/mcp-server/

MCP (Model Context Protocol) server exposing Codex as tools (`codex` and `codex-reply`) over JSON-RPC stdio. Ships as both a library and standalone binary.

## Build & Test
```bash
cargo build -p orbit-code-mcp-server
cargo test -p orbit-code-mcp-server
```

## Architecture

The server runs three concurrent tokio tasks connected by channels: a stdin reader, a message processor, and a stdout writer. The `MessageProcessor` handles MCP protocol negotiation (`initialize`/`initialized`), tool listing (`tools/list`), and tool execution (`tools/call`). Tool calls dispatch to `orbit_code_tool_runner` which creates or continues Codex sessions via `orbit-code-core`'s `ThreadManager`, streams events as MCP notifications, and handles exec/patch approval via MCP elicitation requests.

The `codex` tool starts a new session with configurable model, profile, approval policy, and sandbox mode. The `codex-reply` tool continues an existing session by thread ID. Both tools produce structured MCP responses with thread ID for multi-turn conversation tracking.

## Key Considerations

- The binary uses `arg0_dispatch_or_else` from `orbit-code-arg0` for multi-binary dispatch -- the same binary can serve as different tools based on `argv[0]`.
- Tool parameter schemas are generated via `schemars` at runtime for the MCP `tools/list` response -- changes to `CodexToolCallParam` or `CodexToolCallReplyParam` affect the MCP tool definitions.
- Exec and patch approval use MCP elicitation requests (a protocol extension) -- not all MCP clients support this. The approval flow blocks the tool runner until the client responds.
- Config overrides from tool params are converted from JSON to TOML via `orbit-code-utils-json-to-toml` before merging into the Codex config.
- Test infrastructure uses `mcp_test_support` crate for MCP-specific test helpers.
