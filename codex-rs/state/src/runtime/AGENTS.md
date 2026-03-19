# codex-rs/state/src/runtime/

This file applies to `codex-rs/state/src/runtime/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-state` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-state`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Query implementations for the StateRuntime, organized by domain.

### What this folder does

Each submodule implements the SQLite query logic for a specific domain area of the state database.

### Key files

- `threads.rs` -- thread listing, creation, update, and pagination queries.
- `logs.rs` -- log insertion (batch), querying with filters, and max-ID lookups.
- `backfill.rs` -- backfill state tracking: claiming jobs, updating progress, recording outcomes.
- `agent_jobs.rs` -- CRUD operations for agent jobs and job items.
- `memories.rs` -- memory extraction pipeline: Stage1 job claims, output storage, Phase2 selection and processing.
- `test_support.rs` -- test helpers (cfg(test) only).

### What it plugs into

- These are method implementations on `StateRuntime` (defined in `src/runtime.rs`).
- Called by `codex-core` to persist and query session state.

### Imports from

- `sqlx` for query execution.
- Model types from `src/model/`.
