# codex-rs/tui/

Fullscreen Ratatui-based terminal UI for Orbit Code. Crate name: `orbit-code-tui`.

## Build & Test

```bash
# Build
cargo build -p orbit-code-tui

# Run all unit tests
cargo test -p orbit-code-tui

# Run a specific test
cargo test -p orbit-code-tui -- test_name

# Snapshot tests (insta)
cargo test -p orbit-code-tui           # generates .snap.new files on failure
cargo insta pending-snapshots -p orbit-code-tui  # list pending
cargo insta show -p orbit-code-tui path/to/file.snap.new  # preview
cargo insta accept -p orbit-code-tui   # accept all pending

# VT100 emulator integration tests (not in default features)
cargo test -p orbit-code-tui --features vt100-tests

# Lint
just fix -p orbit-code-tui
just fmt
```

## Architecture

### Entry flow

`main.rs` parses CLI args, then calls `lib.rs::run_main()` which:
1. Loads config (TOML layering, CLI overrides, cloud requirements)
2. Runs onboarding (login, directory trust, welcome screen)
3. Session selection (fresh / resume / fork via `resume_picker`)
4. Enters `App::run()` -- the main event loop

### App event loop (`app.rs`)

`App` is the central state machine. It owns a `ChatWidget` and processes `AppEvent`s from a tokio broadcast channel. Events include keyboard input, agent messages, timer ticks, resize, clipboard, and voice. The loop calls `ChatWidget` methods, then renders via ratatui.

`app.rs` is very large (~310K). Submodules in `app/` handle agent navigation and interactive replay. `app_backtrack.rs` handles conversation undo/rollback.

### Chat pipeline

`ChatWidget` (chatwidget.rs, ~370K) owns the conversation transcript as a vec of `HistoryCell`s plus an active streaming cell. It delegates to:

- **`StreamController`** (`streaming/controller.rs`) -- manages the active agent turn, receives events from core, buffers streaming markdown
- **`chunking`** (`streaming/chunking.rs`) -- splits streaming content into display-ready chunks
- **`commit_tick`** (`streaming/commit_tick.rs`) -- periodic flush of buffered content to the UI

`HistoryCell` (`history_cell.rs`) represents one transcript entry: user message, agent message, tool call, status update, etc.

### Rendering

Markdown is parsed (`markdown.rs`, `markdown_render.rs`) and streamed (`markdown_stream.rs`). Diffs render via `diff_render.rs`. The `render/` module provides syntax highlighting, line utilities, and the `Renderable` trait. Text wrapping lives in `wrapping.rs` and `live_wrap.rs`.

### Input

The `bottom_pane/` module contains the composer (text input), approval overlays, command/skill popups, file search, and the footer status line. `clipboard_paste.rs` and `clipboard_text.rs` handle clipboard integration. `external_editor.rs` launches `$EDITOR`.

### Terminal management

`tui.rs` and `tui/` manage terminal init/restore, alternate screen, raw mode, event streams, frame scheduling, and job control (Ctrl-Z). `custom_terminal.rs` provides inline scrolling and viewport management.

## Key Considerations

### tui/tui_app_server duality

The `tui_app_server/` crate is a near-mirror of this crate that uses app-server JSON-RPC instead of driving core directly. **All UI changes in tui/ MUST be mirrored in tui_app_server/** unless there is a documented reason not to. The two crates share the same module structure, same widget code, same rendering -- they differ in how they talk to the agent backend.

### Snapshot testing

Any UI-affecting change requires updating insta snapshots. Snapshots live in `src/snapshots/` and in various `snapshots/` subdirectories under submodules (`chatwidget/snapshots/`, `status/snapshots/`, etc.). Run `cargo insta accept -p orbit-code-tui` after reviewing `.snap.new` files.

### Feature flags

- `default = ["voice-input"]` -- enables `cpal`/`hound` deps for audio capture. `voice.rs` and `audio_device.rs` are feature-gated.
- `vt100-tests` -- enables VT100 emulator integration tests in `tests/suite/`. These test actual rendered terminal output.
- `debug-logs` -- gates verbose internal TUI logging.

### Module size discipline

`app.rs` (~310K) and `chatwidget.rs` (~370K) are the largest files. The root CLAUDE.md convention says to add new functionality in new modules at ~800 LoC. These files are grandfathered but new features should go into submodules (`app/`, `chatwidget/`).

### Test structure

Integration tests follow the standard pattern: `tests/all.rs` -> `tests/suite/mod.rs` -> individual test files. `tests/test_backend.rs` provides a test-only ratatui backend. Unit tests use sibling `*_tests.rs` files (e.g., `chatwidget/tests.rs`, `status/tests.rs`).
