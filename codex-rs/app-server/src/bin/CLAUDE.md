# app-server/src/bin

## Purpose

Contains auxiliary binary targets for the `codex-app-server` crate, primarily test utilities used by integration tests.

## Key Files

| File | Role |
|------|------|
| `notify_capture.rs` | Test helper binary (`codex-app-server-test-notify-capture`). Atomically writes a payload string to a file using a temp-file-then-rename pattern. Used by integration tests to capture and verify notification delivery without race conditions. |
| `test_notify_capture.rs` | Similar test helper for notification capture, writing a JSON payload atomically via temp file rename. Simplified variant of `notify_capture.rs`. |

## What It Plugs Into

- Both binaries are declared in the crate's `Cargo.toml` as `[[bin]]` targets.
- They are invoked by integration tests (in `tests/suite/`) to verify that the app-server correctly dispatches shell-based notification hooks.

## Imports From

- `anyhow` -- Error handling.
- Standard library (`std::env`, `std::fs`, `std::io`, `std::path`) -- File I/O and argument parsing.

## Exports To

- These are standalone binaries; they do not export library APIs. They produce output files that tests read to verify behavior.
