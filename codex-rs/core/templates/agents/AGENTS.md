# codex-rs/core/templates/agents/

This file applies to `codex-rs/core/templates/agents/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Multi-agent orchestration prompt templates.

### What this folder does

Contains prompt templates used when the Codex agent operates in a multi-agent configuration, providing instructions for how the orchestrating agent should coordinate sub-agents.

### Key files

| File | Purpose |
|------|---------|
| `orchestrator.md` | Instructions for the orchestrator agent role, defining how to decompose tasks and delegate to specialized sub-agents |

### Where it plugs into

- Loaded via `include_str!()` in agent orchestration code
- Used by `crate::agent::role` when configuring orchestrator agents
