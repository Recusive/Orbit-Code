# codex-rs/tui/src/app/

This file applies to `codex-rs/tui/src/app/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

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

Sub-modules of the `App` state machine.

### What this folder does

Contains helper modules that are logically part of the `App` struct defined in `../app.rs` but separated for clarity and testability. These modules handle multi-agent navigation and interactive replay state tracking.

### What it plugs into

- **../app.rs**: The parent `App` state machine imports and owns instances of `AgentNavigationState` and `PendingInteractiveReplayState` from this directory.
- **codex-protocol**: Uses `ThreadId`, `Event`, `EventMsg`, `Op` types for event tracking.

### Key files

| File | Role |
|------|------|
| `agent_navigation.rs` | `AgentNavigationState` -- manages multi-agent picker ordering and traversal. Tracks first-seen spawn order for stable cycling through threads. Keeps picker entry metadata, handles next/previous navigation, and derives user-facing labels for the active agent. |
| `pending_interactive_replay.rs` | `PendingInteractiveReplayState` -- tracks which interactive prompts (exec approvals, patch approvals, MCP elicitations, request_user_input) are still unresolved in the thread-event buffer. Used during thread switching to filter out already-resolved prompts from replayed event snapshots. |

### Design notes

- `AgentNavigationState` maintains traversal in first-seen spawn order (not thread-id sort order). Once a thread is observed it keeps its place in the cycle.
- `PendingInteractiveReplayState` uses both fast lookup sets (by call_id) and turn-indexed queues so `TurnComplete`/`TurnAborted` can clear stale prompts tied to a turn.
