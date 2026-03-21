# codex-rs/tui_app_server/

App-server-backed variant of the Ratatui TUI. Crate name: `orbit-code-tui-app-server`.

## Build & Test

```bash
# Build
cargo build -p orbit-code-tui-app-server

# Run all unit tests
cargo test -p orbit-code-tui-app-server

# Snapshot tests (insta)
cargo test -p orbit-code-tui-app-server
cargo insta pending-snapshots -p orbit-code-tui-app-server
cargo insta accept -p orbit-code-tui-app-server

# VT100 emulator integration tests
cargo test -p orbit-code-tui-app-server --features vt100-tests

# Lint
just fix -p orbit-code-tui-app-server
just fmt
```

## Architecture

### How it differs from tui/

The `tui/` crate drives the agent through `orbit-code-core` directly (in-process). This crate instead wraps an `AppServerSession` that sends JSON-RPC requests and receives streamed notifications via `orbit-code-app-server-client`. The agent backend can run in-process (embedded) or on a remote machine (WebSocket).

Everything else -- the UI widgets, rendering, streaming pipeline, onboarding flows -- is structurally identical. **Changes in tui/ must be mirrored here and vice versa.**

### Entry flow

`main.rs` -> `lib.rs::run_main()` which:
1. Loads config, sets up tracing/telemetry
2. Starts app server: `InProcessAppServerClient` (embedded) or `RemoteAppServerClient` (WebSocket URL)
3. Runs onboarding (welcome, login, directory trust)
4. Session selection (fresh / resume / fork via app-server `thread/*` RPCs)
5. Enters `App::run()` event loop

### App-server session management

`AppServerSession` (`app_server_session.rs`) is a typed wrapper around `AppServerClient` (from `orbit-code-app-server-client`). It provides methods for every JSON-RPC operation: `thread_start`, `turn_start`, `turn_interrupt`, `thread_list`, `thread_fork`, `thread_read`, approvals, model listing, account info, realtime audio, etc. All agent communication goes through this wrapper.

`AppCommand` (`app_command.rs`) wraps protocol `Op`s and provides a command enum for translating UI actions into app-server requests. The `app/` subdir has `app_server_adapter.rs` and `app_server_requests.rs` for the adapter layer between the App event loop and the session.

### Chat pipeline

Same as tui/: `ChatWidget` -> `StreamController` -> chunking -> commit_tick -> `HistoryCell`s. The difference is that events come from app-server notifications instead of core `EventMsg`s.

### Additional modules (not in tui/)

- **`app_server_session.rs`** -- typed JSON-RPC wrapper (the core differentiator)
- **`app_command.rs`** -- command enum bridging UI actions to protocol Ops
- **`local_chatgpt_auth.rs`** -- loads ChatGPT auth tokens from local storage for app-server auth flows
- **`model_catalog.rs`** -- available model listing via app-server API
- **`app/app_server_adapter.rs`** + **`app/app_server_requests.rs`** -- adapter between App and AppServerSession

## Key Considerations

### Mirroring tui/

This is the most important rule: the two crates share nearly identical UI code. When you change a widget, rendering function, or UI behavior in one crate, you must apply the same change to the other. Grep both `tui/src/` and `tui_app_server/src/` to verify.

### Feature flags

Same as tui/: `voice-input` (default), `vt100-tests`, `debug-logs`.

### Snapshot testing

Same workflow as tui/. Snapshots live in `src/snapshots/` and submodule `snapshots/` directories. Use `cargo insta accept -p orbit-code-tui-app-server`.

### Test structure

`tests/all.rs` -> `tests/suite/mod.rs` -> individual test files. Also has `tests/manager_dependency_regression.rs` as a standalone regression test.

### Binary targets

Two binaries: `orbit-code-tui-app-server` (main TUI) and `md-events-app-server` (markdown event debug tool in `src/bin/md-events.rs`).
