# codex-rs/state/src/

This file applies to `codex-rs/state/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-state` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-state`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source code for the `codex-state` crate.

### What this folder does

Implements SQLite-backed state management including thread metadata, structured logging, backfill orchestration, agent jobs, and memories.

### Key files

- `lib.rs` -- module declarations, public re-exports, environment variable and metric constants.
- `runtime.rs` -- `StateRuntime` struct: initializes SQLite connection pools (WAL mode, busy timeout), runs migrations, and coordinates query execution. Declares submodule imports for threads, logs, backfill, agent_jobs, and memories.
- `extract.rs` -- `apply_rollout_item()` transforms `RolloutItem` variants (SessionMeta, TurnContext, EventMsg, ResponseItem) into `ThreadMetadata` mutations. `rollout_item_affects_thread_metadata()` is a fast filter.
- `log_db.rs` -- `tracing_subscriber::Layer` implementation that captures log events and batch-inserts them into the logs SQLite DB via a background task.
- `migrations.rs` -- loads the `STATE_MIGRATOR` and `LOGS_MIGRATOR` from embedded SQL files.
- `paths.rs` -- file modification time utilities.

### Subdirectories

- `model/` -- data model types and database row mappings.
- `runtime/` -- query implementations organized by domain (threads, logs, backfill, agent_jobs, memories).
- `bin/` -- standalone binary (logs_client).

### Imports from

- `codex-protocol` for `ThreadId`, `RolloutItem`, `EventMsg`, etc.
- `sqlx` for SQLite operations.
- `chrono`, `serde`, `tokio`, `tracing`.
