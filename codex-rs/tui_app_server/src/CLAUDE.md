# codex-rs/tui_app_server/src/

Source modules for `orbit-code-tui-app-server`. Same structure as `tui/src/` with app-server-specific additions.

## Module Map

**Core app loop:** `lib.rs` (entry + app-server lifecycle), `app.rs` (event loop), `app_event.rs` / `app_event_sender.rs` (event types + channel), `app_command.rs` (UI actions -> protocol Ops), `app_backtrack.rs` (undo/rollback), `cli.rs` (clap args). `app/` subdir has `agent_navigation.rs`, `pending_interactive_replay.rs`, plus app-server-specific `app_server_adapter.rs` and `app_server_requests.rs`.

**App-server session:** `app_server_session.rs` (typed JSON-RPC wrapper -- the key differentiator from tui/).

**Chat/transcript:** `chatwidget.rs`, `history_cell.rs`, `exec_command.rs`. `chatwidget/` subdir has `interrupts.rs`, `realtime.rs`, `session_header.rs`, `skills.rs`, `tests.rs` (no `agent.rs` -- that logic lives in the app-server adapter instead).

**Streaming pipeline:** `streaming/` -- `controller.rs`, `chunking.rs`, `commit_tick.rs`. Same as tui/.

**Input/composer:** `bottom_pane/` (same structure as tui/), `clipboard_paste.rs`, `clipboard_text.rs`, `external_editor.rs`, `insert_history.rs`, `mention_codec.rs`, `slash_command.rs`.

**Rendering:** `render/`, `markdown.rs` + `markdown_render.rs` + `markdown_stream.rs`, `diff_render.rs`, `wrapping.rs`, `live_wrap.rs`, `line_truncation.rs`, `shimmer.rs`, `style.rs`, `color.rs`. Same as tui/.

**Terminal:** `tui.rs` + `tui/`, `custom_terminal.rs`, `ascii_animation.rs` + `frames.rs`. Same as tui/.

**Session:** `resume_picker.rs`, `session_log.rs`, `cwd_prompt.rs`.

**Onboarding/config:** `onboarding/`, `model_migration.rs`, `oss_selection.rs`, `collaboration_modes.rs`, `debug_config.rs`, `theme_picker.rs`.

**Status/overlays:** `status/`, `notifications/`, `pager_overlay.rs`, `tooltips.rs`, `status_indicator_widget.rs`, `update_prompt.rs`, `updates.rs`.

**Public widgets:** `public_widgets/` exports `ComposerInput`.

## Key Differences from tui/src/

- **`app_server_session.rs`** -- all agent communication goes through this typed JSON-RPC wrapper instead of calling core directly.
- **`app_command.rs`** -- translates UI commands to protocol Ops for the app-server. tui/ does not have this (it calls core Ops directly).
- **`local_chatgpt_auth.rs`** -- loads ChatGPT auth tokens from local disk for app-server auth flows.
- **`model_catalog.rs`** -- fetches available models via app-server API.
- **`app/app_server_adapter.rs`** + **`app/app_server_requests.rs`** -- adapter layer between App and AppServerSession (tui/ has no equivalent).
- **`chatwidget/` has no `agent.rs`** -- agent interaction logic lives in the app-server adapter instead.

## Patterns

Same as tui/src/: sibling `*_tests.rs` files, feature-gated `voice.rs`/`audio_device.rs`, snapshot dirs at multiple levels, `exec_cell/` model/render split.
