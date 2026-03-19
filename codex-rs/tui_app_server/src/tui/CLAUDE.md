# codex-rs/tui_app_server/src/tui/

Terminal management submodules for the TUI event loop.

## What this folder does

Contains the internal plumbing for terminal I/O: the crossterm event stream adapter, frame rate limiting, frame request scheduling, and Unix job control (Ctrl+Z suspend/resume). These modules are used by the `Tui` struct (defined in `../tui.rs`) to manage the terminal lifecycle.

## What it plugs into

- **../tui.rs**: The parent `Tui` struct owns and coordinates these submodules. `Tui` is the top-level terminal abstraction used by `App::run()`.
- **../app.rs**: `App` interacts with `Tui` via `TuiEvent`s from the event stream and `FrameRequester` for scheduling redraws.
- **crossterm**: Raw terminal events are received from `crossterm::event::EventStream`.
- **ratatui**: Frame rendering goes through the ratatui `Terminal` backend.

## Key files

| File | Role |
|------|------|
| `event_stream.rs` | `TuiEventStream` / `EventBroker` -- adapts the crossterm event stream into `TuiEvent`s (key events, mouse, resize, focus, paste). Handles event broadcasting to multiple subscribers. |
| `frame_rate_limiter.rs` | Frame rate limiter -- enforces a minimum interval between frames (`MIN_FRAME_INTERVAL`) to prevent excessive redraws while maintaining responsiveness. |
| `frame_requester.rs` | `FrameRequester` -- allows any widget to request a UI redraw. Coalesces multiple requests within the same frame interval. |
| `job_control.rs` | `SuspendContext` (Unix only) -- handles Ctrl+Z (SIGTSTP) suspend and resume by saving/restoring terminal state. |

## Imports from

- `crossterm::event` -- `EventStream`, `KeyEvent`, `KeyEventKind`, etc.
- `tokio_stream` -- async stream utilities.
- `tokio::sync::broadcast` -- event broadcasting.
- `crate::custom_terminal` -- custom ratatui terminal wrapper.
- `crate::notifications` -- notification backend for the Tui.

## Exports to

- **../tui.rs**: `TuiEventStream`, `EventBroker`, `FrameRequester`, `SuspendContext`.
- **crate::app** / **crate::chatwidget** / **crate::bottom_pane**: `FrameRequester` for scheduling redraws; `TuiEvent` for event processing.
