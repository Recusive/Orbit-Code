# codex-rs/tui/tests/suite/

This file applies to `codex-rs/tui/tests/suite/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-tui` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.
- Any user-visible TUI change needs matching snapshot coverage. Mirror behavior in the sibling TUI implementation when the same feature exists there.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-tui`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo insta pending-snapshots -p codex-tui`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Individual integration test modules for the TUI.

### What this folder does

Contains the test modules aggregated by `../all.rs` into a single integration test binary. Each module tests a specific aspect of TUI behavior.

### What it plugs into

- **../all.rs**: Declares all modules here as `mod suite;` which then includes each test file.
- **codex-tui**: Tests exercise the crate's public and internal APIs.
- **../test_backend.rs**: VT100-based tests use the `VT100Backend` for terminal emulation.

### Key files

| File | Role |
|------|------|
| `mod.rs` | Module root; declares all test sub-modules (`model_availability_nux`, `no_panic_on_startup`, `status_indicator`, `vt100_history`, `vt100_live_commit`). |
| `no_panic_on_startup.rs` | Smoke test ensuring the TUI binary starts without panicking. Spawns the actual `codex-tui` binary and verifies it exits cleanly. |
| `vt100_history.rs` | VT100 emulator-based tests for history line insertion and scrolling. Verifies that `insert_history_lines()` correctly renders content in both inline and alternate-screen viewports. Gated behind `vt100-tests` feature. |
| `vt100_live_commit.rs` | VT100 emulator-based tests for live streaming commit animation rendering. Verifies the streaming pipeline's visual output. Gated behind `vt100-tests` feature. |
| `status_indicator.rs` | Tests for the status indicator widget rendering. |
| `model_availability_nux.rs` | Tests for the model availability NUX (new user experience) flow. |
