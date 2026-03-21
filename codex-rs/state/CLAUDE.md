# codex-rs/state/

SQLite-backed persistent state: thread metadata, structured log storage, backfill orchestration, agent jobs, and memories.

## Build & Test
```bash
cargo build -p orbit-code-state
cargo test -p orbit-code-state
```

## Architecture

`StateRuntime` is the primary entry point -- it manages two SQLite databases (state DB for threads/backfill/jobs/memories, logs DB for structured log events), runs migrations on startup, and provides query methods organized by domain. Thread metadata is extracted from JSONL rollout files via `apply_rollout_item()` which transforms protocol-level events into structured table rows.

The `log_db` module implements a `tracing_subscriber::Layer` that captures log events and batch-inserts them into the logs DB via a background tokio task. A standalone `logs_client` binary (`src/bin/`) provides CLI-based log tailing.

## Key Considerations

- Both databases use WAL mode with busy timeout for concurrent read access -- but writes are not safe across multiple processes.
- Migrations live in `migrations/` (state DB) and `logs_migrations/` (logs DB) as SQL files. Adding a migration requires updating `BUILD.bazel` if using `sqlx::migrate!`.
- `apply_rollout_item()` is the core data extraction function -- it must handle all `RolloutItem` variants. Adding a new variant to `orbit-code-protocol` requires updating the extraction logic here.
- The `SQLITE_HOME_ENV` environment variable overrides the default SQLite database location.
- `ThreadMetadataBuilder` uses a builder pattern for incremental construction from multiple rollout items.
