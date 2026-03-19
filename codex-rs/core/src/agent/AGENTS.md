# codex-rs/core/src/agent/

This file applies to `codex-rs/core/src/agent/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Multi-agent control, lifecycle management, and role configuration for the Codex agent system.

### What this folder does

This module manages the spawning, tracking, and lifecycle of AI agents within a Codex session. It handles:

- **Agent control** (`control.rs`): Spawning sub-agents with unique nicknames, forking parent history, managing agent lifecycle (start, status watching, shutdown). Uses `AgentControl` as the main coordination struct shared across all agents in a user session.
- **Guards** (`guards.rs`): Rate-limiting and depth-limiting for multi-agent spawning. Tracks active agents per session, assigns unique nicknames from a pool, and enforces thread spawn depth limits.
- **Roles** (`role.rs`): Applies agent-role configuration layers (built-in or user-defined) on top of an existing session config. Roles are loaded with the same config machinery as `config.toml` and inserted at session-flag precedence.
- **Status** (`status.rs`): Derives agent status (`Running`, `Completed`, `Interrupted`, `Errored`, `Shutdown`) from emitted protocol events.
- **Builtins** (`builtins/`): Built-in agent role definitions as TOML files.

### Key files

| File | Purpose |
|------|---------|
| `mod.rs` | Module declarations and re-exports: `AgentControl`, `AgentStatus`, guards |
| `control.rs` | `AgentControl` -- spawns sub-agents, manages nicknames, watches status |
| `guards.rs` | `Guards` -- enforces agent count limits and depth limits per session |
| `role.rs` | `apply_role_to_config()` -- resolves and applies role config layers |
| `status.rs` | `agent_status_from_event()` -- maps `EventMsg` to `AgentStatus` |
| `agent_names.txt` | Pool of agent nicknames used for sub-agent identification |

### Imports from

- `codex_protocol` -- `AgentStatus`, `EventMsg`, `ThreadId`, `SessionSource`
- `crate::config` -- `Config`, `AgentRoleConfig`, `ConfigOverrides`
- `crate::codex_thread` -- `ThreadConfigSnapshot`
- `crate::rollout` -- `RolloutRecorder`
- `crate::error` -- `CodexErr`, `Result`

### Exports to

- `crate::codex` -- uses `AgentControl` for multi-agent orchestration
- `crate::tools::handlers::multi_agents` -- uses spawn/guards for the `spawn_agent` tool
- `crate::state` -- `AgentControl` is held in `SessionServices`
