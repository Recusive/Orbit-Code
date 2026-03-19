# codex-rs/core/templates/agents/

Multi-agent orchestration prompt templates.

## What this folder does

Contains prompt templates used when the Codex agent operates in a multi-agent configuration, providing instructions for how the orchestrating agent should coordinate sub-agents.

## Key files

| File | Purpose |
|------|---------|
| `orchestrator.md` | Instructions for the orchestrator agent role, defining how to decompose tasks and delegate to specialized sub-agents |

## Where it plugs into

- Loaded via `include_str!()` in agent orchestration code
- Used by `crate::agent::role` when configuring orchestrator agents
