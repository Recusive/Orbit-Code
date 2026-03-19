# codex-rs/tui/src/status/

Status output formatting and display adapters for the TUI.

## What this folder does

Turns protocol-level snapshots into stable display structures used by the `/status` command output and footer/status-line helpers. Keeps rendering concerns out of transport-facing code. The `rate_limits` sub-module is the main integration point for status-line usage-limit items.

## What it plugs into

- **../chatwidget.rs**: The chat widget uses status formatters for displaying rate limits, token counts, and directory info in the footer and status line.
- **../app.rs**: `App` calls `new_status_output()` / `new_status_output_with_rate_limits()` to build the `/status` card.
- **codex-protocol**: Reads `RateLimitSnapshot`, `RateLimitWindow`, `CreditsSnapshot`, `TokenUsage` protocol types.

## Key files

| File | Role |
|------|------|
| `mod.rs` | Module root; re-exports key types and functions (`new_status_output`, `format_directory_display`, `format_tokens_compact`, `RateLimitSnapshotDisplay`, `RateLimitWindowDisplay`). |
| `card.rs` | `new_status_output()` and `new_status_output_with_rate_limits()` -- builds the full `/status` output card as a composite `HistoryCell`. Includes model, sandbox policy, approval policy, directory, token usage, rate limits, credits, reasoning details, network access, thread ID/name, and version info. |
| `account.rs` | `StatusAccountDisplay` enum -- distinguishes between ChatGPT (email + plan) and API key account display modes. |
| `format.rs` | `FieldFormatter` -- utility for consistently formatting label-value rows with alignment. Includes `line_display_width()`, `push_label()`, and `truncate_line_to_width()` helpers. |
| `helpers.rs` | Helper functions: `format_directory_display()` (shortens paths), `format_tokens_compact()` (human-readable token counts), `compose_account_display()`, `format_reset_timestamp()`. |
| `rate_limits.rs` | Rate limit display shaping: converts `RateLimitSnapshot` protocol payloads into `StatusRateLimitRow`s with bar visualizations, percentage labels, and reset-time labels. Handles stale data detection and credits display. |
| `tests.rs` | Test suite for status output rendering. |

## Sub-directories

| Directory | Purpose |
|-----------|---------|
| `snapshots/` | Insta test snapshots for status output tests. |
