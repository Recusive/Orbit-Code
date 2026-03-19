# codex-rs/tui_app_server/src/chatwidget/

This file applies to `codex-rs/tui_app_server/src/chatwidget/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

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

Submodules for the `ChatWidget` -- the main chat surface in the TUI.

### What this folder does

Contains supplementary modules for the `ChatWidget` struct (defined in `../chatwidget.rs`). These handle interrupt semantics, realtime audio session management, session header rendering, and skill display logic.

### What it plugs into

- **../chatwidget.rs**: The parent `ChatWidget` struct uses these modules for interrupt handling, realtime mode, header display, and skill integration.
- **../app.rs**: `App` owns the `ChatWidget` and drives it from the event loop.
- **../bottom_pane/**: The chat widget owns the bottom pane and delegates input/rendering.

### Key files

| File | Role |
|------|------|
| `interrupts.rs` | Interrupt handling logic -- tracks Ctrl+C press counts, manages the "press again to quit" flow, and coordinates agent interruption vs. UI dismissal. |
| `realtime.rs` | Realtime audio session management -- start/stop voice capture, audio frame forwarding, and microphone state tracking. |
| `session_header.rs` | Renders the session header bar showing model name, thread info, collaboration mode, and session status. |
| `skills.rs` | Skill metadata display and formatting for the chat widget. |
| `tests.rs` | Unit tests for `ChatWidget` behavior. |

### Imports from

- `crate::app_event` / `crate::app_event_sender` -- event types and sender.
- `crate::bottom_pane` -- `BottomPane`, approval overlays.
- `crate::history_cell` -- `HistoryCell` for transcript entries.
- `crate::streaming` -- `StreamController` for active streaming.
- `codex_app_server_protocol` -- turn and thread types.

### Exports to

- **crate::app**: `ChatWidget` types like `ExternalEditorState`, `ReplayKind`, `ThreadInputState` are re-exported for `App` use.
