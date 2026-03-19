# codex-rs/async-utils/

This file applies to `codex-rs/async-utils/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-async-utils` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-async-utils`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Crate: `codex-async-utils` -- Async utility extensions for Tokio-based code.

### What this crate does

Provides ergonomic extensions for working with async futures and cancellation tokens. The primary feature is the `OrCancelExt` trait, which allows any `Future` to be raced against a `CancellationToken`.

### Main types and functions

- `CancelErr` -- Error type returned when a future is cancelled (unit variant `Cancelled`)
- `OrCancelExt` trait -- Extension trait for all `Future + Send` types:
  - `.or_cancel(token: &CancellationToken) -> Result<Output, CancelErr>` -- Races the future against the token; returns `Ok(output)` if the future completes first, or `Err(CancelErr::Cancelled)` if the token fires first

### What it plugs into

- Used throughout the workspace wherever async operations need to be cancellable (agent turns, network requests, etc.)

### Imports from / exports to

**Dependencies:**
- `async-trait` -- For async trait definitions
- `tokio` -- Runtime (macros, rt, time)
- `tokio-util` -- Provides `CancellationToken`

**Exports:**
- `CancelErr` and `OrCancelExt` are the public API

### Key files

- `Cargo.toml` -- Crate manifest
- `src/lib.rs` -- Single-file implementation with the trait, blanket impl, and tests
