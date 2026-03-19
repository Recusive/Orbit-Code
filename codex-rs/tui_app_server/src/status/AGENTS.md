# codex-rs/tui_app_server/src/status/

This file applies to `codex-rs/tui_app_server/src/status/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

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

Status output formatting and display adapters for the TUI `/status` command.

### What this folder does

Converts protocol-level snapshots (account info, rate limits, token usage, session metadata) into stable display structures used by the `/status` command output and the footer/status-line helpers. Keeps rendering concerns separate from transport-facing code. The `rate_limits` submodule is the main integration point for status-line usage-limit items.

### What it plugs into

- **../app.rs**: `App` uses this module to generate the `/status` output card and footer status indicators.
- **../bottom_pane/status_line_setup.rs**: Footer status line consumes rate limit display data.
- **codex_protocol**: Token usage, rate limit snapshots, account plan types.
- **codex_core::config**: Session metadata (model, sandbox, approval policy, cwd, etc.).

### Key files

| File | Role |
|------|------|
| `mod.rs` | Module root; re-exports public types and declares submodules. |
| `account.rs` | `StatusAccountDisplay` enum -- represents the account type (ChatGPT with email/plan, or API key). |
| `card.rs` | `new_status_output()` / `new_status_output_with_rate_limits()` -- builds the full `/status` output as a `CompositeHistoryCell` with session info, account, model, sandbox, approval policy, token usage, and rate limits. |
| `format.rs` | `FieldFormatter` -- formatting helpers for status card fields (label/value pairs, line width measurement, truncation). |
| `helpers.rs` | Utility functions: `format_directory_display()`, `format_tokens_compact()`, `compose_account_display()`. |
| `rate_limits.rs` | `RateLimitSnapshotDisplay` / `RateLimitWindowDisplay` -- converts raw rate limit window snapshots into local-time labels with stale/available/missing classification. |
| `tests.rs` | Unit tests for status formatting and rate limit display. |

### Imports from

- `crate::history_cell` -- `HistoryCell`, `CompositeHistoryCell`, `PlainHistoryCell`, border helpers.
- `crate::version` -- `CODEX_CLI_VERSION`.
- `codex_protocol` -- `ThreadId`, `TokenUsage`, `SandboxPolicy`, `AskForApproval`, rate limit types, account/plan types.
- `codex_core::config::Config` -- session configuration.
- `chrono` -- local-time formatting for rate limit windows.

### Exports to

- **crate::app** / **crate::bottom_pane**: `StatusAccountDisplay`, `new_status_output_with_rate_limits`, `format_directory_display`, `format_tokens_compact`, `RateLimitSnapshotDisplay`, `RateLimitWindowDisplay`, `rate_limit_snapshot_display_for_limit`.
