# codex-rs/core/src/tools/handlers/multi_agents/

This file applies to `codex-rs/core/src/tools/handlers/multi_agents/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Tool handlers for multi-agent orchestration (spawn, communicate, lifecycle).

### What this folder does

Implements the tool handlers that enable the AI agent to spawn and manage sub-agents. These tools allow the primary agent to delegate work to specialized child agents.

### Key files

| File | Purpose |
|------|---------|
| `spawn.rs` | `spawn_agent` handler -- creates a new sub-agent with a specified role and initial message. Supports forking parent history. |
| `send_input.rs` | `send_input` handler -- sends a message to a running sub-agent |
| `wait.rs` | `wait_agent` handler -- waits for a sub-agent to complete and returns its final output |
| `resume_agent.rs` | `resume_agent` handler -- resumes a paused/completed sub-agent |
| `close_agent.rs` | `close_agent` handler -- shuts down a sub-agent and cleans up its resources |

### Imports from

- `crate::agent` -- `AgentControl`, `apply_role_to_config`, `SpawnAgentOptions`, depth guards
- `crate::tools::handlers` -- `ToolHandler` trait, argument parsing, `ToolInvocation`
- `crate::codex` -- `Session`, `TurnContext`

### Exports to

- `crate::tools::handlers::multi_agents` (parent `multi_agents.rs`) -- re-exports handler types for registration
- `crate::tools::registry` -- registered as available tools
