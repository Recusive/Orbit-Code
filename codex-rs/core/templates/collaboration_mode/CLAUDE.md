# codex-rs/core/templates/collaboration_mode/

Collaboration mode preset definitions.

## What this folder does

Defines the available collaboration mode presets that control how the agent interacts with the user. Each preset provides different system prompt instructions optimizing for a specific workflow.

## Key files

| File | Purpose |
|------|---------|
| `default.md` | Default collaboration mode -- balanced between autonomous execution and user interaction |
| `execute.md` | Execute mode -- agent focuses on completing tasks autonomously with minimal interaction |
| `pair_programming.md` | Pair programming mode -- agent works interactively, explaining decisions and seeking feedback |
| `plan.md` | Plan mode -- agent creates detailed plans before executing, with user approval at each step |

## Where it plugs into

- Loaded via `include_str!()` in `crate::models_manager::collaboration_mode_presets`
- Selected based on `collaboration_mode` config setting
- Injected into the system prompt during session initialization
