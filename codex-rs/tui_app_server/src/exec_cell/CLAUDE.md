# codex-rs/tui_app_server/src/exec_cell/

Data model and rendering for command execution cells in the chat transcript.

## What this folder does

Implements the `ExecCell` type that represents a grouped set of shell command executions in the TUI transcript. Each cell can contain one or more `ExecCall` entries (single commands or "exploring" groups of related read/list/search commands). The module handles both the data model (tracking command state, output, timing) and the rendering logic (formatting command lines, output truncation, spinners, diff summaries).

## What it plugs into

- **../chatwidget.rs**: `ChatWidget` uses `ExecCell` as a variant of `HistoryCell` in the chat transcript. It routes progress and completion events into the right cell by `call_id`.
- **../app.rs**: `App` creates exec cells when the agent executes shell commands and updates them with output/completion events from the app server.
- **../history_cell.rs**: `ExecCell` is stored inside `HistoryCell::Exec` variants.

## Key files

| File | Role |
|------|------|
| `mod.rs` | Module root; re-exports `ExecCell`, `ExecCall`, `CommandOutput`, and render functions. |
| `model.rs` | Data model -- `ExecCell` struct (holds a vec of `ExecCall`s), `ExecCall` struct (command, parsed form, output, timing), `CommandOutput` struct (exit code, aggregated output, formatted output). |
| `render.rs` | Rendering logic -- `output_lines()` formats exec cell output for display, `new_active_exec_command()` creates the active-command header, `spinner()` generates the in-progress spinner, `TOOL_CALL_MAX_LINES` caps output display. |

## Imports from

- `codex_protocol::parse_command::ParsedCommand` -- parsed shell command structure.
- `codex_protocol::protocol::ExecCommandSource` -- whether the command was agent-initiated or user-initiated.
- `crate::render::highlight` -- syntax highlighting for command output.
- `crate::diff_render::DiffSummary` -- file diff rendering.

## Exports to

- **crate::chatwidget** / **crate::app**: `ExecCell`, `ExecCall`, `CommandOutput`, `output_lines`, `OutputLinesParams`, `TOOL_CALL_MAX_LINES`, `new_active_exec_command`, `spinner`.
