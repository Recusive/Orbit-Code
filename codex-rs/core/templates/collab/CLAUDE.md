# codex-rs/core/templates/collab/

Experimental collaboration mode prompt templates.

## What this folder does

Contains experimental prompt templates for collaborative interaction modes where the agent works more interactively with the user.

## Key files

| File | Purpose |
|------|---------|
| `experimental_prompt.md` | Experimental collaboration prompt defining interactive pair-programming style behavior |

## Where it plugs into

- Loaded via `include_str!()` when the `Collab` feature flag is enabled
- Used by collaboration mode configuration in `crate::models_manager::collaboration_mode_presets`
