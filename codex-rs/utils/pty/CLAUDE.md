# codex-rs/utils/pty/

Crate `codex-utils-pty` -- PTY and pipe-based process spawning with cross-platform process group management.

## What this folder does

Provides two process spawning backends -- PTY-attached (for interactive shells) and pipe-based (for non-interactive commands) -- with unified `ProcessHandle` and `SpawnedProcess` abstractions. Handles process group management, terminal resizing, stdin/stdout/stderr multiplexing via Tokio channels, and reliable cross-platform process termination including descendant cleanup.

## Key types and functions

- `ProcessHandle` -- handle for interacting with a spawned process: write stdin, kill, resize PTY, check exit status
- `SpawnedProcess` -- bundle of `ProcessHandle` plus split stdout/stderr `mpsc::Receiver`s and exit `oneshot::Receiver`
- `TerminalSize` -- rows/cols specification for PTY operations
- `spawn_pty_process` -- spawn with a PTY attached for interactive use
- `spawn_pipe_process` / `spawn_pipe_process_no_stdin` -- spawn with regular pipes
- `combine_output_receivers` -- merge stdout/stderr into a single `broadcast::Receiver`
- `conpty_supported()` -- check ConPTY availability on Windows
- `DEFAULT_OUTPUT_BYTES_CAP` -- 1 MiB default output buffer cap

## Imports from

- `anyhow` -- error handling
- `portable-pty` -- cross-platform PTY abstraction
- `tokio` -- async I/O, channels, process spawning
- `libc` (Unix) -- process group, session, and signal management
- `winapi` (Windows) -- process termination, ConPTY

## Exports to

Consumed by `codex-core` for executing shell commands (both interactive PTY sessions and non-interactive pipe commands).

## Key files

- `Cargo.toml` -- crate metadata with platform-specific dependencies
- `src/lib.rs` -- module declarations, public re-exports, type aliases
- `src/process.rs` -- `ProcessHandle`, `SpawnedProcess`, `TerminalSize`, `PtyHandles`, `combine_output_receivers`
- `src/pipe.rs` -- pipe-based process spawning (stdin piped or null)
- `src/pty.rs` -- PTY-based process spawning with portable-pty and raw Unix PTY fallback
- `src/process_group.rs` -- process group helpers: `detach_from_tty`, `set_process_group`, `kill_process_group`, `set_parent_death_signal`
- `src/tests.rs` -- integration tests
- `src/win/` -- Windows-specific ConPTY implementation (vendored from WezTerm)
