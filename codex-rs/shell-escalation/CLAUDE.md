# codex-rs/shell-escalation/

Unix shell-escalation protocol and `codex-execve-wrapper` binary.

## What this folder does

Implements the exec-interception protocol that allows a sandboxed shell to escalate commands to run outside the sandbox. A patched bash invokes `codex-execve-wrapper` on every `exec()` call. The wrapper sends the proposed command to the escalation server over a Unix domain socket, and the server responds with Run (execute in sandbox), Escalate (forward file descriptors and execute outside sandbox), or Deny (reject the command).

## What it plugs into

- Used by `codex-core` for sandbox command execution on Unix.
- The `EscalateServer` is instantiated by the runtime when running shell commands with sandbox escalation.
- The `codex-execve-wrapper` binary is set as the `EXEC_WRAPPER` / `BASH_EXEC_WRAPPER` environment variable for the patched shell.

## Imports from

- `codex-protocol` -- `EscalationPermissions`, `Permissions`, approval types.
- `codex-utils-absolute-path` -- path normalization.
- `socket2` -- Unix socket pair creation.
- `tokio`, `tokio-util` -- async runtime, cancellation tokens.
- `clap` -- CLI argument parsing for the wrapper binary.
- `libc` -- low-level Unix syscalls (execv, dup2, etc.).

## Exports to

- `EscalateServer` -- the server that listens for escalation requests.
- `EscalationSession` -- session handle with environment overlay for the shell process.
- `EscalationPolicy` trait -- callers implement this to decide Run/Escalate/Deny.
- `ShellCommandExecutor` trait -- callers implement this for process spawning.
- `ExecParams`, `ExecResult`, `PreparedExec` -- execution parameter/result types.
- `Stopwatch` -- pausable timer for command timeouts.
- `main_execve_wrapper` -- entrypoint for the wrapper binary.
- `ESCALATE_SOCKET_ENV_VAR` -- environment variable name for the socket FD.

## Key files

- `Cargo.toml` -- crate manifest; builds `codex-execve-wrapper` binary.
- `README.md` -- protocol documentation and patched-bash build instructions.
- `src/lib.rs` -- conditional compilation gate (Unix only) and public re-exports.
- `src/unix/` -- Unix implementation.
