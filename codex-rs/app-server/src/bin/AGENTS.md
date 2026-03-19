# codex-rs/app-server/src/bin/

This file applies to `codex-rs/app-server/src/bin/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-app-server` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-app-server`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### Purpose

Contains auxiliary binary targets for the `codex-app-server` crate, primarily test utilities used by integration tests.

### Key Files

| File | Role |
|------|------|
| `notify_capture.rs` | Test helper binary (`codex-app-server-test-notify-capture`). Atomically writes a payload string to a file using a temp-file-then-rename pattern. Used by integration tests to capture and verify notification delivery without race conditions. |
| `test_notify_capture.rs` | Similar test helper for notification capture, writing a JSON payload atomically via temp file rename. Simplified variant of `notify_capture.rs`. |

### What It Plugs Into

- Both binaries are declared in the crate's `Cargo.toml` as `[[bin]]` targets.
- They are invoked by integration tests (in `tests/suite/`) to verify that the app-server correctly dispatches shell-based notification hooks.

### Imports From

- `anyhow` -- Error handling.
- Standard library (`std::env`, `std::fs`, `std::io`, `std::path`) -- File I/O and argument parsing.

### Exports To

- These are standalone binaries; they do not export library APIs. They produce output files that tests read to verify behavior.
