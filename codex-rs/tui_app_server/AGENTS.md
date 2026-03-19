# codex-rs/tui_app_server/

This file applies to `codex-rs/tui_app_server/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-tui-app-server` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.
- Any user-visible TUI change needs matching snapshot coverage. Mirror behavior in the sibling TUI implementation when the same feature exists there.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-tui-app-server`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo insta pending-snapshots -p codex-tui-app-server`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

The `codex-tui-app-server` crate: an app-server-backed variant of the fullscreen Ratatui TUI for the Codex CLI.

### What this folder does

This is a parallel implementation of the `codex-tui` crate (`codex-rs/tui/`) that communicates through the app-server protocol instead of driving `codex-core` directly. It provides the same chat-style terminal interface (streaming markdown, tool approval, session resume/fork, onboarding, voice input) but routes all agent interactions through either an embedded in-process app server or a remote WebSocket-connected app server. This architecture enables IDE integrations and remote-server deployments while reusing the same TUI surface.

### What it plugs into

- **codex-app-server-client**: Client library for connecting to the app server (both `InProcessAppServerClient` and `RemoteAppServerClient`).
- **codex-app-server-protocol**: JSON-RPC request/response/notification types for the app-server wire protocol (`ThreadStart`, `TurnStart`, `TurnInterrupt`, approvals, etc.).
- **codex-core**: Config loading, auth, feature flags, path utilities, and session metadata -- used during startup but not for live agent communication.
- **codex-protocol**: Shared protocol types (`ThreadId`, `SandboxPolicy`, `AskForApproval`, rollout items).
- **codex-cli**: The top-level binary dispatches into this crate when app-server mode is active.
- **ratatui / crossterm**: Terminal rendering and keyboard input.

### Key difference from codex-rs/tui/

The `tui/` crate drives the agent through `codex-core` directly (in-process event loop). This crate instead wraps an `AppServerSession` that sends JSON-RPC requests and receives streamed notifications, allowing the agent backend to run in a separate process or on a remote machine.

### Main exports

- `run_main(cli, arg0_paths, loader_overrides, remote)` -- entry point; loads config, runs onboarding, starts the ratatui event loop with an app-server backend.
- `Cli` -- clap-derived CLI argument struct.
- `AppExitInfo` / `ExitReason` -- return types describing how the session ended.
- `ComposerInput` / `ComposerAction` -- public reusable text-input widget.
- `render_markdown_text()` -- public markdown rendering helper.

### Key files

| File | Role |
|------|------|
| `Cargo.toml` | Crate manifest; depends on `codex-app-server-client`, `codex-app-server-protocol`, plus ~40 workspace crates. |
| `src/main.rs` | Binary entry point; parses CLI and calls `run_main()`. |
| `src/lib.rs` | Library root; declares modules, manages app-server lifecycle (embedded/remote), runs onboarding and the ratatui loop. |
| `src/cli.rs` | `Cli` struct with clap definitions (prompt, model, sandbox, approval policy, etc.). |
| `src/app.rs` | `App` -- central state machine processing TUI events and app-server notifications. |
| `src/app_server_session.rs` | `AppServerSession` -- typed wrapper around the app-server client with methods for every RPC. |
| `src/tui.rs` | Terminal lifecycle (init, restore, set_modes) and the `Tui` wrapper. |
| `src/chatwidget.rs` | `ChatWidget` -- main chat surface; owns history cells, streaming cell, and bottom pane. |
| `src/frames.rs` | Compile-time embedded ASCII animation frames (10 variants, 36 frames each). |
| `frames/` | Raw ASCII art frame text files for loading animations. |
| `tests/` | Integration tests including VT100 emulator-based rendering tests. |
| `styles.md` | Design reference for TUI styling conventions. |
| `tooltips.txt` | Tooltip content data. |

### Architecture

```
main.rs -> lib.rs::run_main()
  -> config loading, tracing, telemetry
  -> start_app_server() (Embedded or Remote)
  -> onboarding (welcome, login, trust directory)
  -> session selection (fresh / resume / fork via app-server thread RPCs)
  -> run_ratatui_app()
       -> App::run() event loop
            -> AppServerSession (JSON-RPC requests/notifications)
            -> ChatWidget (history + active cell + bottom pane)
            -> streaming pipeline (StreamController -> chunking -> commit_tick)
            -> terminal rendering via ratatui
```

### Features

- `default = ["voice-input"]` -- enables audio capture/transcription.
- `vt100-tests` -- enables VT100 emulator-based integration tests.
- `debug-logs` -- gates verbose TUI debug logging.
