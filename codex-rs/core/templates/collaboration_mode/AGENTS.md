# codex-rs/core/templates/collaboration_mode/

This file applies to `codex-rs/core/templates/collaboration_mode/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Collaboration mode preset definitions.

### What this folder does

Defines the available collaboration mode presets that control how the agent interacts with the user. Each preset provides different system prompt instructions optimizing for a specific workflow.

### Key files

| File | Purpose |
|------|---------|
| `default.md` | Default collaboration mode -- balanced between autonomous execution and user interaction |
| `execute.md` | Execute mode -- agent focuses on completing tasks autonomously with minimal interaction |
| `pair_programming.md` | Pair programming mode -- agent works interactively, explaining decisions and seeking feedback |
| `plan.md` | Plan mode -- agent creates detailed plans before executing, with user approval at each step |

### Where it plugs into

- Loaded via `include_str!()` in `crate::models_manager::collaboration_mode_presets`
- Selected based on `collaboration_mode` config setting
- Injected into the system prompt during session initialization
