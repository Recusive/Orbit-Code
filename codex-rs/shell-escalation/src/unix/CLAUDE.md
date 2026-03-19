# codex-rs/shell-escalation/src/unix/

Unix implementation of the shell-escalation protocol.

## What this folder does

Implements the full exec-interception protocol: the client side (wrapper binary), the server side (escalation listener), the wire protocol, the policy trait, socket primitives, and a pausable stopwatch for command timeouts.

## Key files

- `mod.rs` -- module declarations, re-exports, and ASCII art protocol flow diagrams showing the escalation and non-escalation paths.
- `escalate_protocol.rs` -- wire protocol types: `EscalateRequest` (file, argv, workdir, env), `EscalateResponse`, `EscalateAction` (Run/Escalate/Deny), `EscalationDecision`, `EscalationExecution` (Unsandboxed/TurnDefault/Permissions), `SuperExecMessage` (FD forwarding), `SuperExecResult`. Also defines environment variable names (`CODEX_ESCALATE_SOCKET`, `EXEC_WRAPPER`, `BASH_EXEC_WRAPPER`).
- `escalation_policy.rs` -- `EscalationPolicy` async trait with `determine_action()` method. Callers implement this to decide whether commands should run in sandbox, be escalated, or be denied.
- `escalate_server.rs` -- `EscalateServer` creates escalation sessions. `EscalationSession` holds the environment overlay and background task. The server listens for datagram handshakes, spawns per-request stream handlers, applies the policy, and either sends Run or forwards FDs for Escalate. Also defines `ShellCommandExecutor` trait, `ExecParams`, `ExecResult`, `PreparedExec`.
- `escalate_client.rs` -- `run_shell_escalation_execve_wrapper()` implements the client side: reads the socket FD from env, sends the EscalateRequest, handles Run (calls `execv()`), Escalate (duplicates and sends stdio FDs, waits for exit code), and Deny (prints error, exits 1).
- `execve_wrapper.rs` -- CLI parsing with `clap` and the `main_execve_wrapper()` async entrypoint.
- `socket.rs` -- `AsyncSocket` (SOCK_STREAM with length-prefixed JSON framing and SCM_RIGHTS FD passing) and `AsyncDatagramSocket` (SOCK_DGRAM for the handshake). Handles non-blocking I/O via `tokio::io::unix::AsyncFd`.
- `stopwatch.rs` -- `Stopwatch` with pause/resume support for command timeouts. Generates a `CancellationToken` that fires when the (unpaused) elapsed time exceeds the limit.

## Imports from

- `socket2` for raw socket operations.
- `tokio`, `tokio-util` for async I/O and cancellation.
- `libc` for `execv`, `dup2`, `SCM_RIGHTS`, `CMSG_*` macros.
- `codex-protocol` for `EscalationPermissions`.
- `codex-utils-absolute-path` for path resolution.

## Exports to

- Everything is re-exported through `src/lib.rs` to the parent crate.
