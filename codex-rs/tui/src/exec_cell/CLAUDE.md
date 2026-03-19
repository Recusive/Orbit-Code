# codex-rs/tui/src/exec_cell/

Data model and rendering for grouped exec-call history cells.

## What this folder does

Defines the `ExecCell` data model and its rendering logic for displaying shell command executions in the TUI transcript. An `ExecCell` can represent a single command or an "exploring" group of related read/list/search commands that are visually collapsed together.

## What it plugs into

- **../chatwidget.rs**: `ChatWidget` creates `ExecCell` instances when the agent executes shell commands and inserts them into the transcript as `HistoryCell` entries.
- **../history_cell.rs**: `ExecCell` produces `HistoryCell` instances via its rendering functions.
- **codex-protocol**: Uses `ParsedCommand`, `ExecCommandSource` for command metadata.

## Key files

| File | Role |
|------|------|
| `mod.rs` | Module root; re-exports `ExecCell`, `ExecCall`, `CommandOutput`, rendering functions, and constants. |
| `model.rs` | Data model: `ExecCall` (a single command invocation with call_id, command, parsed form, output, timing), `ExecCell` (a group of `ExecCall`s), and `CommandOutput` (exit code, stdout/stderr, formatted output). Handles grouping logic for "exploring" cells. |
| `render.rs` | Rendering logic: converts `ExecCell` into ratatui `Line`s for display. Handles syntax-highlighted bash commands, truncated output previews (default 5 lines, 50 for user shell tools), spinners for in-progress commands, duration formatting, and interaction input previews. |

## Constants

- `TOOL_CALL_MAX_LINES = 5` -- default max output lines shown for tool calls.
- User shell tool calls show up to 50 lines.
- Max interaction preview: 80 characters.

## Imports from

- `crate::render::highlight` for bash syntax highlighting
- `crate::shimmer` for loading animations
- `crate::wrapping` for adaptive text wrapping
- `codex-ansi-escape`, `codex-shell-command`, `codex-utils-elapsed` for output formatting
