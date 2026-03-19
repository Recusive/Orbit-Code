# codex-rs/core/templates/memories/

Memory pipeline prompt templates for extraction and consolidation.

## What this folder does

Provides the system prompts and input templates for the two-phase memory pipeline that learns from past sessions.

## Key files

| File | Purpose |
|------|---------|
| `stage_one_system.md` | Phase 1 system prompt: instructs the extraction agent to identify key learnings, patterns, and preferences from a session rollout |
| `stage_one_input.md` | Phase 1 input template: structures how rollout data is presented to the extraction agent |
| `consolidation.md` | Phase 2 system prompt: instructs the consolidation agent to merge, deduplicate, and prioritize raw memories into a coherent summary |
| `read_path.md` | Template for the memory read path -- how stored memories are presented to the agent during normal sessions |

## Where it plugs into

- Loaded via `include_str!()` in `crate::memories`
- Phase 1 prompts used in `crate::memories::phase1`
- Phase 2 prompts used in `crate::memories::phase2`
- Read path template used when injecting memories into regular session context
