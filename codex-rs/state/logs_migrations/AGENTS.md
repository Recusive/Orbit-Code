# codex-rs/state/logs_migrations/

This file applies to `codex-rs/state/logs_migrations/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-state` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-state`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

SQLite migration files for the dedicated logs database.

### What this folder does

Contains SQL migration scripts that define and evolve the schema of the logs SQLite database (`logs.db`). Migrations are embedded at compile time by `sqlx::migrate!()`.

### Key files

- `0001_logs.sql` -- initial logs table schema.
- `0002_logs_feedback_log_body.sql` -- adds feedback log body support.
