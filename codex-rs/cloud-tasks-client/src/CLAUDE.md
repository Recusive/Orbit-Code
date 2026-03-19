# codex-rs/cloud-tasks-client/src/

Source directory for the `codex-cloud-tasks-client` crate.

## What this folder does

Contains the `CloudBackend` trait definition, shared types, and feature-gated implementations for cloud task operations.

## Key files

| File | Role |
|------|------|
| `lib.rs` | Module declarations; public re-exports of all types and feature-gated clients |
| `api.rs` | `CloudBackend` async trait with methods for listing, getting details, applying diffs, creating tasks, listing attempts; type definitions: `TaskId`, `TaskStatus`, `TaskSummary`, `DiffSummary`, `TurnAttempt`, `AttemptStatus`, `ApplyOutcome`, `ApplyStatus`, `TaskText`, `TaskListPage`, `CreatedTask`, `CloudTaskError` |
| `http.rs` | `HttpClient` (requires `online` feature) -- wraps `codex-backend-client::Client`; maps backend response models to crate types; handles diff extraction from multiple turn formats |
| `mock.rs` | `MockClient` (requires `mock` feature) -- returns deterministic mock data with environment-based variation for testing |
