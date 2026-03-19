# codex-rs/utils/pty/src/

This file applies to `codex-rs/utils/pty/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-utils-pty` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-utils-pty`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source directory for the `codex-utils-pty` crate.

### Key files

- `lib.rs` -- module declarations and public API re-exports; defines `DEFAULT_OUTPUT_BYTES_CAP` (1 MiB)
- `process.rs` -- core abstractions:
  - `ProcessHandle` -- wraps writer channel, child killer, reader/writer/wait task handles, PTY handles; provides `writer_sender()`, `has_exited()`, `exit_code()`, `resize()`, `close_stdin()`, `request_terminate()`, `terminate()`; implements `Drop` for cleanup
  - `SpawnedProcess` -- bundles `ProcessHandle` with `stdout_rx`, `stderr_rx`, `exit_rx`
  - `TerminalSize` -- rows/cols with default 24x80
  - `PtyHandles` / `PtyMasterHandle` -- keep PTY file descriptors alive
  - `combine_output_receivers` -- merges stdout/stderr via `tokio::select!` into a broadcast channel
- `pipe.rs` -- pipe-based spawning:
  - `spawn_process` / `spawn_process_no_stdin` -- async functions using `tokio::process::Command`
  - Unix `pre_exec`: detaches from TTY, sets parent death signal (Linux), closes inherited FDs
  - `PipeChildTerminator` -- kills process group on Unix, single process on Windows
- `pty.rs` -- PTY-based spawning:
  - `spawn_process` -- uses `portable-pty` on most platforms, raw `openpty` on Unix when preserving inherited FDs
  - `spawn_process_preserving_fds` -- Unix-only path using raw `libc::openpty`, `setsid`, `TIOCSCTTY`
  - `close_inherited_fds_except` -- closes non-stdio FDs in `/dev/fd` except preserved ones
  - `conpty_supported()` -- delegates to Windows ConPTY check
- `process_group.rs` -- OS-specific process group management:
  - `set_process_group()` -- `setpgid(0, 0)` on Unix
  - `detach_from_tty()` -- `setsid()` on Unix (falls back to `setpgid`)
  - `kill_process_group` / `kill_process_group_by_pid` / `terminate_process_group` -- SIGKILL/SIGTERM to process groups
  - `set_parent_death_signal` -- Linux-only `prctl(PR_SET_PDEATHSIG)` with race detection
  - All functions are no-ops on non-Unix platforms
- `tests.rs` -- integration tests
- `win/` -- Windows ConPTY implementation (see `win/` subdirectory)
