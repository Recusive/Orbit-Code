# codex-rs/state/src/runtime/

Query implementations for the StateRuntime, organized by domain.

## What this folder does

Each submodule implements the SQLite query logic for a specific domain area of the state database.

## Key files

- `threads.rs` -- thread listing, creation, update, and pagination queries.
- `logs.rs` -- log insertion (batch), querying with filters, and max-ID lookups.
- `backfill.rs` -- backfill state tracking: claiming jobs, updating progress, recording outcomes.
- `agent_jobs.rs` -- CRUD operations for agent jobs and job items.
- `memories.rs` -- memory extraction pipeline: Stage1 job claims, output storage, Phase2 selection and processing.
- `test_support.rs` -- test helpers (cfg(test) only).

## What it plugs into

- These are method implementations on `StateRuntime` (defined in `src/runtime.rs`).
- Called by `codex-core` to persist and query session state.

## Imports from

- `sqlx` for query execution.
- Model types from `src/model/`.
