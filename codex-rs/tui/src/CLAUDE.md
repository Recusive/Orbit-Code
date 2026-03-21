# codex-rs/tui/src/

Source modules for `orbit-code-tui`. Flat layout with subdirectories for complex subsystems.

## Module Map

**Core app loop:** `lib.rs` (entry + bootstrap), `app.rs` (event loop state machine), `app_event.rs` / `app_event_sender.rs` (event types + channel), `app_backtrack.rs` (undo/rollback), `cli.rs` (clap args), `app_server_tui_dispatch.rs` (delegates to app-server TUI when active). `app/` subdir has `agent_navigation.rs` and `pending_interactive_replay.rs`.

**Chat/transcript:** `chatwidget.rs` (main chat surface + history), `history_cell.rs` (individual transcript entries), `exec_command.rs` (shell command display). `chatwidget/` subdir has `agent.rs`, `interrupts.rs`, `realtime.rs`, `session_header.rs`, `skills.rs`, `tests.rs`.

**Streaming pipeline:** `streaming/` -- `controller.rs` (StreamController), `chunking.rs` (content splitting), `commit_tick.rs` (periodic flush to UI).

**Input/composer:** `bottom_pane/` (composer, approval overlays, popups, footer, file search, skill/command pickers), `clipboard_paste.rs`, `clipboard_text.rs`, `external_editor.rs`, `insert_history.rs`, `mention_codec.rs`, `slash_command.rs`.

**Rendering:** `render/` (syntax highlighting, line utils, Renderable trait), `markdown.rs` + `markdown_render.rs` + `markdown_stream.rs` (markdown pipeline), `diff_render.rs`, `wrapping.rs`, `live_wrap.rs`, `line_truncation.rs`, `shimmer.rs`, `style.rs`, `color.rs`.

**Terminal:** `tui.rs` + `tui/` (terminal lifecycle, event stream, frame scheduling, job control), `custom_terminal.rs` (inline scrolling/viewport), `ascii_animation.rs` + `frames.rs` (loading spinners).

**Session:** `resume_picker.rs` (resume/fork picker), `session_log.rs` (event logging), `cwd_prompt.rs` (directory selection on resume).

**Onboarding/config:** `onboarding/` (welcome, login, directory trust), `model_migration.rs`, `oss_selection.rs`, `collaboration_modes.rs`, `debug_config.rs`, `theme_picker.rs`.

**Status/overlays:** `status/` (account, rate limits, session card), `notifications/` (OSC9, BEL), `pager_overlay.rs`, `tooltips.rs`, `status_indicator_widget.rs`, `update_prompt.rs`, `updates.rs`.

**Public widgets:** `public_widgets/` exports `ComposerInput` for use by external crates.

## Patterns

- Tests use sibling `*_tests.rs` files (e.g., `chatwidget/tests.rs`, `markdown_render_tests.rs`, `status/tests.rs`).
- `voice.rs` and `audio_device.rs` are feature-gated behind `voice-input` (and excluded on Linux).
- `exec_cell/` has its own `model.rs` / `render.rs` split for exec tool call rendering.
- Snapshot dirs (`snapshots/`) appear at multiple levels: `src/snapshots/`, `chatwidget/snapshots/`, `status/snapshots/`, `onboarding/snapshots/`, `bottom_pane/snapshots/`, `render/snapshots/`.
