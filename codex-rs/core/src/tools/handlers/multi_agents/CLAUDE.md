# codex-rs/core/src/tools/handlers/multi_agents/

Tool handlers for multi-agent orchestration (spawn, communicate, lifecycle).

## What this folder does

Implements the tool handlers that enable the AI agent to spawn and manage sub-agents. These tools allow the primary agent to delegate work to specialized child agents.

## Key files

| File | Purpose |
|------|---------|
| `spawn.rs` | `spawn_agent` handler -- creates a new sub-agent with a specified role and initial message. Supports forking parent history. |
| `send_input.rs` | `send_input` handler -- sends a message to a running sub-agent |
| `wait.rs` | `wait_agent` handler -- waits for a sub-agent to complete and returns its final output |
| `resume_agent.rs` | `resume_agent` handler -- resumes a paused/completed sub-agent |
| `close_agent.rs` | `close_agent` handler -- shuts down a sub-agent and cleans up its resources |

## Imports from

- `crate::agent` -- `AgentControl`, `apply_role_to_config`, `SpawnAgentOptions`, depth guards
- `crate::tools::handlers` -- `ToolHandler` trait, argument parsing, `ToolInvocation`
- `crate::codex` -- `Session`, `TurnContext`

## Exports to

- `crate::tools::handlers::multi_agents` (parent `multi_agents.rs`) -- re-exports handler types for registration
- `crate::tools::registry` -- registered as available tools
