# codex-rs/state/src/

SQLite state management implementation: runtime, data models, extraction, and log layer.

## Module Layout

- **runtime** (`runtime.rs` + `runtime/`) -- `StateRuntime` struct: DB pool initialization, migration execution. Submodules in `runtime/` organize queries by domain (threads, logs, backfill, agent_jobs, memories)
- **model/** -- Data model types and database row mappings: `ThreadMetadata`, `ThreadMetadataBuilder`, `ThreadsPage`, `LogEntry`, `LogRow`, `AgentJob`, `BackfillState`
- **extract** (`extract.rs`) -- `apply_rollout_item()`: transforms `RolloutItem` variants into `ThreadMetadata` mutations; `rollout_item_affects_thread_metadata()` fast filter
- **log_db** (`log_db.rs`) -- `tracing_subscriber::Layer` that captures and batch-inserts log events into SQLite via background task
- **migrations** (`migrations.rs`) -- SQLx migration runner setup for state and logs databases
- **bin/** -- `logs_client` standalone binary for CLI log tailing
