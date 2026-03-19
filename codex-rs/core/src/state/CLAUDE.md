# codex-rs/core/src/state/

Session services, session state, and turn state management.

## What this folder does

Provides the internal state containers for a Codex session, separated into three layers:

### SessionServices (`service.rs`)
Long-lived, shared service handles that persist for the lifetime of a session. Includes:
- MCP connection manager and startup cancellation
- Unified exec process manager
- Analytics, hooks, rollout recorder
- Shell configuration and snapshots
- Execution policy, auth, models manager
- Tool approval store, skill/plugin managers
- File watcher, agent control, network proxy/approval
- Session telemetry

### SessionState (`session.rs`)
Mutable session-scoped state:
- `SessionConfiguration` -- active configuration snapshot
- `ContextManager` -- conversation history
- Rate limit tracking, dependency env vars
- Previous turn settings for model continuity
- Startup prewarm handle
- Active connector selection
- Granted permissions (session-wide)

### Turn state (`turn.rs`)
Per-turn mutable state:
- `ActiveTurn` -- tracks running tasks with their cancellation tokens and done notifications
- `RunningTask` -- individual task handle (kind, cancellation, join handle, turn context, timer)
- `TaskKind` -- Regular, Review, or Compact
- `TurnState` -- pending approvals, tool call counts, token usage tracking, granted turn permissions

## Key files

| File | Purpose |
|------|---------|
| `mod.rs` | Module declarations and re-exports |
| `service.rs` | `SessionServices` -- long-lived service handles |
| `session.rs` | `SessionState` -- mutable session-scoped state |
| `session_tests.rs` | Tests for session state |
| `turn.rs` | `ActiveTurn`, `RunningTask`, `TaskKind`, `TurnState` |

## Imports from

- `crate::codex` -- `Session`, `TurnContext`, `PreviousTurnSettings`
- `crate::context_manager` -- `ContextManager`
- `crate::tools::sandboxing` -- `ApprovalStore`
- `crate::unified_exec` -- `UnifiedExecProcessManager`
- Many service types from across `codex-core`

## Exports to

- `crate::codex` -- `Session` holds `SessionServices` and `SessionState`
- `crate::tasks` -- `ActiveTurn`, `RunningTask`, `TaskKind` used by task orchestration
- `crate::tools` -- `SessionServices` accessed during tool execution
