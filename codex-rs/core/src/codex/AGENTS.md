# codex-rs/core/src/codex/

This file applies to `codex-rs/core/src/codex/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Session rollout reconstruction logic for resume and fork operations.

### What this folder does

This subdirectory module handles rebuilding conversation history from persisted rollout files. When a user resumes or forks an existing thread, the rollout reconstruction logic replays recorded events to reconstruct:

- The conversation history (`Vec<ResponseItem>`)
- Previous turn settings (model, reasoning effort, etc.) for continuity
- Reference context items for efficient model-visible state diffing

### Key files

| File | Purpose |
|------|---------|
| `rollout_reconstruction.rs` | `RolloutReconstruction` struct and replay logic: walks rollout items in reverse to find the newest surviving compaction checkpoint, applies rollback semantics, and extracts turn settings |
| `rollout_reconstruction_tests.rs` | Tests for reconstruction edge cases (rollback past compaction, multi-turn replay) |

### Key concepts

- **Replay segments**: Groups of rollout items belonging to the same turn, identified by turn ID.
- **Replacement history**: A compaction checkpoint that replaces all older history items.
- **Rollback turns**: The reconstruction skips the newest N user-turn boundaries to implement undo/rollback.
- **Reference context item**: Baseline for model-visible settings diffing, tracking what the model last saw.

### Imports from

- `crate::codex` (parent) -- `Session`, `PreviousTurnSettings`, `TurnContextItem`
- `codex_protocol` -- `ResponseItem`, `RolloutItem`, `EventMsg`

### Exports to

- `crate::codex::Session` -- called during `reconstruct_history_from_rollout()` for resume/fork flows
