# codex-rs/cli/src/debug_sandbox/

This file applies to `codex-rs/cli/src/debug_sandbox/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-cli` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-cli`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

macOS-specific sandbox debugging helpers (PID tracking and denial logging).

### What this folder does

Provides tools for debugging macOS Seatbelt sandbox behavior: tracking all descendant PIDs of a sandboxed process via `kqueue` and `proc_listchildpids`, and streaming macOS `log stream` output to capture and parse sandbox denial messages.

### Where it plugs in

- Used by `debug_sandbox.rs` (parent module) when `--log-denials` is passed to `codex sandbox macos`
- `DenialLogger` captures sandbox denials during a sandboxed child process execution
- `PidTracker` monitors all forked child/grandchild PIDs so denial logs can be filtered to relevant processes

### Imports from

- `libc` -- `kqueue`, `kevent`, `proc_listchildpids` (macOS-specific)
- `tokio` -- async process spawning for `log stream`, BufReader for stdout
- `regex_lite` -- parsing sandbox denial log messages
- `serde_json` -- parsing ndjson log stream output

### Key files

| File | Role |
|------|------|
| `pid_tracker.rs` | `PidTracker` -- uses `kqueue` with `EVFILT_PROC` (NOTE_FORK/NOTE_EXEC/NOTE_EXIT) to recursively track all descendant PIDs; `list_child_pids` wraps `proc_listchildpids` |
| `seatbelt.rs` | `DenialLogger` -- spawns `log stream --style ndjson` filtered to sandbox subsystem; `SandboxDenial` struct; `parse_message` extracts process name, PID, and denied capability from log lines |
