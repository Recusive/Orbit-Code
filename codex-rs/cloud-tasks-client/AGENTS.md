# codex-rs/cloud-tasks-client/

This file applies to `codex-rs/cloud-tasks-client/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-cloud-tasks-client` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-cloud-tasks-client`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Backend client abstraction for Codex Cloud tasks operations.

### What this folder does

Defines the `CloudBackend` async trait for cloud task operations (list, get details, apply diffs, create tasks, list attempts) and provides two implementations: `HttpClient` (real HTTP via `codex-backend-client`) and `MockClient` (in-memory mock for testing). Also provides shared types for task status, summaries, diffs, and apply outcomes.

### Where it plugs in

- Used by `codex-cloud-tasks` for all backend communication
- `HttpClient` wraps `codex-backend-client::Client` with cloud-task-specific logic
- `MockClient` provides deterministic test data
- Uses `codex-git` for diff application (`apply_git_patch`)
- Feature-gated: `online` enables `HttpClient`, `mock` enables `MockClient`

### Imports from

- `codex-backend-client` (optional, `online` feature) -- HTTP client for backend API
- `codex-git` -- `apply_git_patch` for applying diffs locally
- `async-trait` -- trait definition
- `chrono` -- timestamps
- `diffy` -- diff parsing/formatting
- `serde` / `serde_json` -- serialization

### Exports to

Public API from `lib.rs`:

- `CloudBackend` trait -- async methods: `list_tasks`, `task_details`, `apply_diff`, `create_task`, `list_attempts`
- `HttpClient` (feature `online`) -- real HTTP implementation
- `MockClient` (feature `mock`) -- mock implementation
- Types: `TaskId`, `TaskStatus`, `TaskSummary`, `DiffSummary`, `TurnAttempt`, `AttemptStatus`, `ApplyOutcome`, `ApplyStatus`, `TaskText`, `TaskListPage`, `CreatedTask`, `CloudTaskError`, `Result`

### Key files

| File | Role |
|------|------|
| `Cargo.toml` | Crate manifest with `online` and `mock` features |
| `src/lib.rs` | Module declarations and public re-exports |
| `src/api.rs` | `CloudBackend` trait definition and all shared types |
| `src/http.rs` | `HttpClient` -- wraps `codex-backend-client` for task CRUD, diff extraction, apply, and attempt listing |
| `src/mock.rs` | `MockClient` -- deterministic mock data for testing with environment-based variation |
