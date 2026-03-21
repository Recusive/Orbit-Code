# codex-rs/shell-escalation/

Unix shell-escalation protocol that allows a sandboxed shell to escalate commands for execution outside the sandbox. Includes the `codex-execve-wrapper` binary.

## Build & Test
```bash
cargo build -p orbit-code-shell-escalation
cargo test -p orbit-code-shell-escalation
```

## Architecture

A patched bash invokes `codex-execve-wrapper` on every `exec()` call. The wrapper sends the proposed command to the `EscalateServer` over a Unix domain socket. The server evaluates the command against an `EscalationPolicy` and responds with Run (execute in sandbox), Escalate (forward file descriptors and execute outside sandbox), or Deny (reject). The `EscalationSession` provides the environment overlay needed for the patched shell, including the socket FD in `ESCALATE_SOCKET_ENV_VAR`.

## Key Considerations
- Unix-only: the entire implementation is behind `#[cfg(unix)]` -- this crate is a no-op on Windows
- The `codex-execve-wrapper` binary is set as `EXEC_WRAPPER` / `BASH_EXEC_WRAPPER` environment variable for the patched shell
- Uses raw `libc` syscalls (`execv`, `dup2`, file descriptor management) in the wrapper binary
- Socket pairs are created via `socket2` for reliable Unix domain socket communication
- `Stopwatch` provides pausable timing for command timeouts -- paused during user approval prompts
- Callers implement both `EscalationPolicy` (decision logic) and `ShellCommandExecutor` (process spawning) traits
- The patched bash is a separate build (not included in this repo) -- see `README.md` for build instructions
