# codex-rs/tui_app_server/tests/suite/

This file applies to `codex-rs/tui_app_server/tests/suite/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-tui-app-server` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.
- Any user-visible TUI change needs matching snapshot coverage. Mirror behavior in the sibling TUI implementation when the same feature exists there.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-tui-app-server`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo insta pending-snapshots -p codex-tui-app-server`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Individual integration test modules for the TUI.

### What this folder does

Contains the actual integration test implementations, aggregated by the parent `all.rs` binary. Each module tests a specific area of TUI functionality, often using the VT100 emulator backend to verify rendered terminal output.

### What it plugs into

- **../all.rs**: The `suite` module is included from `all.rs`, which compiles all tests into a single binary.
- **../test_backend.rs**: VT100-based backend (when `vt100-tests` feature is enabled) for rendering verification.
- **../fixtures/**: Test modules may load fixture files for replay scenarios.

### Key files

| File | Role |
|------|------|
| `mod.rs` | Module root; declares all test submodules. |
| `no_panic_on_startup.rs` | Regression test verifying the TUI starts without panicking under various configurations. |
| `status_indicator.rs` | Tests for the status indicator widget rendering. |
| `vt100_history.rs` | VT100 emulator tests for chat history rendering (feature-gated behind `vt100-tests`). |
| `vt100_live_commit.rs` | VT100 emulator tests for live streaming commit animation (feature-gated behind `vt100-tests`). |
| `model_availability_nux.rs` | Tests for model availability new-user-experience prompts. |
| `manager_dependency_regression.rs` | Regression test for dependency manager interactions. |

### Imports from

- `codex_tui_app_server` -- crate library for test setup.
- `codex_core` / `codex_cli` / `codex_utils_pty` -- test infrastructure.
- `vt100` (dev-dependency) -- terminal emulator for rendering tests.

### Exports to

- Tests only; no exports to other crates.
