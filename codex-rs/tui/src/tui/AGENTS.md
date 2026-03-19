# codex-rs/tui/src/tui/

This file applies to `codex-rs/tui/src/tui/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-tui` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.
- Any user-visible TUI change needs matching snapshot coverage. Mirror behavior in the sibling TUI implementation when the same feature exists there.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-tui`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo insta pending-snapshots -p codex-tui`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Terminal subsystem modules for the TUI.

### What this folder does

Contains the low-level terminal management infrastructure: the crossterm event stream plumbing, frame rate limiting, frame draw scheduling, and Unix job control (Ctrl+Z suspend/resume). These modules support the `Tui` wrapper defined in `../tui.rs`.

### What it plugs into

- **../tui.rs**: The parent `Tui` struct uses `EventBroker`, `TuiEventStream`, `FrameRequester`, and (on Unix) `SuspendContext` from this directory to manage the terminal lifecycle.
- **../app.rs**: `App` consumes `TuiEvent`s from the event stream and uses `FrameRequester` to schedule redraws.
- All animation/status widgets use `FrameRequester` to request redraws.

### Key files

| File | Role |
|------|------|
| `event_stream.rs` | `EventBroker` -- holds the shared crossterm event stream. `TuiEventStream` wraps a draw subscription plus the event broker and maps crossterm events into `TuiEvent`. Supports drop/recreate of the crossterm stream to fully relinquish stdin during external editor or subprocess launches. Defines `EventSource` trait (with `CrosstermEventSource` real impl and `FakeEventSource` for tests). |
| `frame_rate_limiter.rs` | `FrameRateLimiter` -- clamps draw notifications to a maximum of 120 FPS (~8.33ms interval) to avoid wasted rendering work. Pure helper with no async dependencies. |
| `frame_requester.rs` | `FrameRequester` -- lightweight cloneable handle for scheduling future frame draws. Internally spawns a `FrameScheduler` actor task that coalesces multiple requests into a single notification on a broadcast channel. Follows the actor pattern from "Actors with Tokio". |
| `job_control.rs` | (Unix only) `SuspendContext` -- coordinates Ctrl+Z suspend/resume. Records which resume path to take (realign inline viewport vs. restore alternate screen), caches the inline cursor row, and provides `prepare_resume_action()` for viewport adjustments after SIGCONT. |

### Key constants

- `MIN_FRAME_INTERVAL` = ~8.33ms (120 FPS) -- exported as `TARGET_FRAME_INTERVAL` by `../tui.rs`.
