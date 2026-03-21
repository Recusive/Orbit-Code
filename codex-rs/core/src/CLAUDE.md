# codex-rs/core/src/

Source tree for `orbit-code-core`. Entry point is `lib.rs`.

## Module Categories

**Session & agent loop** -- `codex.rs` (Session/agent loop), `orbit_code_thread.rs` (CodexThread wrapper), `thread_manager.rs` (ThreadManager), `orbit_code_delegate.rs` (lifecycle callbacks), `agent/` (agent control, guards, roles, status), `tasks/` (turn task types), `state/` (session/turn state)

**Config & features** -- `config/` (Config struct, builder, schema, types, permissions, profiles), `config_loader/` (layered loading from system/user/project/runtime), `features.rs` + `features/` (Feature enum, feature flags)

**Tools & execution** -- `tools/` (registry, router, orchestrator, handlers, runtimes, sandboxing), `unified_exec/` (interactive PTY management), `exec.rs` (command execution), `sandboxing/` (Seatbelt/seccomp/landlock wrappers), `seatbelt.rs`, `landlock.rs`

**Auth & security** -- `auth.rs` (types, re-exports), `auth/` (manager, persistence, recovery, storage), `anthropic_auth/` (Anthropic OAuth, token refresh), `guardian/` (automated approval)

**External integrations** -- `client.rs` + `client_common.rs` (HTTP client for Responses API), `mcp/` + `mcp_connection_manager.rs` + `mcp_tool_call.rs` (MCP servers), `connectors.rs` (app connectors), `skills/` + `plugins/` + `instructions/` (skills, plugins, AGENTS.md)

**Persistence & context** -- `rollout/` (session persistence, discovery, indexing), `context_manager/` (conversation history, token accounting), `memories/` (extraction, consolidation), `compact.rs` (context compaction), `message_history.rs`

## Key Patterns

**Sibling test files**: Unit tests live in `foo_tests.rs` alongside `foo.rs`, not inline `#[cfg(test)] mod tests`. This applies throughout the entire src/ tree.

**Session -> tools flow**: `codex.rs` (Session) processes model responses, extracts tool calls, passes them to `tools/router.rs` (ToolRouter), which dispatches to `tools/orchestrator.rs` for approval/sandbox/execute, which delegates to `tools/handlers/` and `tools/runtimes/`.

**Event emission**: Session emits `EventMsg` variants through an `async_channel::Sender`. Consumers (CodexThread/ThreadManager) receive on the other end. The session never calls back into consumers directly.
