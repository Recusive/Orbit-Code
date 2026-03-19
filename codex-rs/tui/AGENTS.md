# codex-rs/tui/

This file applies to `codex-rs/tui/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-tui` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.
- Any user-visible TUI change needs matching snapshot coverage. Mirror behavior in the sibling TUI implementation when the same feature exists there.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-tui`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo insta pending-snapshots -p codex-tui`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

The `codex-tui` crate: the fullscreen Ratatui-based terminal user interface for the Codex CLI.

### What this folder does

This is the primary interactive TUI for the Codex CLI. It provides a chat-style interface where users converse with an AI agent, approve tool executions, view streaming markdown output, manage sessions (resume/fork), and configure settings -- all within a terminal. Built on `ratatui` and `crossterm`, the TUI runs in the terminal's alternate screen buffer (configurable) and drives the agent via `codex-core`.

### What it plugs into

- **codex-core**: Business logic engine -- session management, config loading, auth, agent orchestration, MCP, and the thread/event protocol.
- **codex-protocol**: Wire types (`Op`, `EventMsg`, `Event`, `TokenUsage`, `SandboxPolicy`, etc.) that flow between UI and agent.
- **codex-cli**: The top-level binary dispatches into this crate's `run_main()` for interactive sessions.
- **codex-tui-app-server**: Alternative TUI backend for app-server mode; the binary delegates to it when appropriate.
- **ratatui / crossterm**: Terminal rendering and input handling.

### Main exports

- `run_main(cli, arg0_paths, loader_overrides)` -- the main entry point that loads config, runs onboarding, and starts the ratatui event loop.
- `Cli` -- clap-derived CLI argument struct.
- `AppExitInfo` / `ExitReason` -- return types describing how the session ended.
- `should_use_app_server_tui()` -- feature-gate check for app-server mode.
- `ComposerInput` / `ComposerAction` -- public reusable text-input widget (used by external crates like `codex-cloud-tasks`).
- `render_markdown_text()` -- public markdown rendering helper.

### Key files

| File | Role |
|------|------|
| `Cargo.toml` | Crate manifest; declares dependencies on ~40 workspace crates plus ratatui, crossterm, syntect, etc. |
| `src/main.rs` | Binary entry point; parses CLI, dispatches to `run_main()` or app-server mode. |
| `src/lib.rs` | Library root; declares all modules, runs onboarding, config loading, tracing setup, and launches the ratatui app. |
| `src/cli.rs` | `Cli` struct with clap argument definitions (prompt, model, sandbox, approval policy, etc.). |
| `src/app.rs` | `App` -- the central application state machine that processes events and drives the UI. |
| `src/tui.rs` | Terminal lifecycle (init, restore, set_modes) and the `Tui` wrapper around the ratatui terminal. |
| `src/chatwidget.rs` | `ChatWidget` -- the main chat surface; owns history cells, active streaming cell, and bottom pane. |
| `src/frames.rs` | Compile-time embedded ASCII animation frames for loading spinners (10 animation variants). |
| `frames/` | Raw ASCII art frame files (36 frames per variant, 10 variants). |
| `tests/` | Integration tests including VT100 emulator-based rendering tests. |
| `styles.md` | Design reference for TUI styling conventions. |
| `tooltips.txt` | Tooltip content data. |

### Architecture

```
main.rs -> lib.rs::run_main() -> run_ratatui_app()
  -> onboarding (login, trust directory, welcome)
  -> session selection (fresh / resume / fork)
  -> App::run() event loop
       -> ChatWidget (history + active cell + bottom pane)
       -> streaming pipeline (StreamController -> chunking -> commit_tick)
       -> terminal rendering via ratatui
```

### Features

- `default = ["voice-input"]` -- enables audio capture/transcription.
- `vt100-tests` -- enables VT100 emulator-based integration tests.
- `debug-logs` -- gates verbose TUI debug logging.
