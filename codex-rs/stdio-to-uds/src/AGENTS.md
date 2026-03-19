# codex-rs/stdio-to-uds/src/

This file applies to `codex-rs/stdio-to-uds/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-stdio-to-uds` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-stdio-to-uds`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source code for the `codex-stdio-to-uds` crate.

### What this folder does

Contains the library implementation and binary entrypoint for the stdio-to-UDS adapter.

### Key files

- `lib.rs` -- `run(socket_path)`: connects to a Unix domain socket, spawns a thread to copy socket data to stdout, copies stdin to the socket in the calling thread, performs a half-close on the write side, and waits for the stdout thread to finish.
- `main.rs` -- CLI binary that validates a single command-line argument (socket path) and calls `run()`.

### Imports from

- `std::os::unix::net::UnixStream` (Unix) or `uds_windows::UnixStream` (Windows).
- `anyhow` for error handling.
