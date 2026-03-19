# codex-rs/state/migrations/

This file applies to `codex-rs/state/migrations/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-state` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-state`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

SQLite migration files for the main state database.

### What this folder does

Contains SQL migration scripts that define and evolve the schema of the state SQLite database (`state.db`). Migrations are embedded at compile time by `sqlx::migrate!()`. Currently at version 20.

### Key files (chronological)

- `0001_threads.sql` -- initial threads table.
- `0002_logs.sql` -- logs table in state DB.
- `0003_logs_thread_id.sql` -- adds thread_id to logs.
- `0004_thread_dynamic_tools.sql` -- dynamic tools per thread.
- `0005_threads_cli_version.sql` -- CLI version tracking.
- `0006_memories.sql` -- memories table.
- `0007_threads_first_user_message.sql` -- first user message column.
- `0008_backfill_state.sql` -- backfill tracking table.
- `0009_stage1_outputs_rollout_slug.sql` -- rollout slug for stage1 outputs.
- `0010_logs_process_id.sql` -- process ID in logs.
- `0011_logs_partition_prune_indexes.sql` -- partition pruning indexes.
- `0012_logs_estimated_bytes.sql` -- estimated byte tracking.
- `0013_threads_agent_nickname.sql` -- agent nickname.
- `0014_agent_jobs.sql` -- agent jobs table.
- `0015_agent_jobs_max_runtime_seconds.sql` -- max runtime for jobs.
- `0016_memory_usage.sql` -- memory usage tracking.
- `0017_phase2_selection_flag.sql` -- Phase2 selection flag.
- `0018_phase2_selection_snapshot.sql` -- Phase2 selection snapshot.
- `0019_thread_dynamic_tools_defer_loading.sql` -- deferred tool loading.
- `0020_threads_model_reasoning_effort.sql` -- model reasoning effort.
