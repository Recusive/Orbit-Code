# codex-rs/shell-escalation/src/

Source for the `orbit-code-shell-escalation` crate -- Unix exec-interception protocol with escalation server and wrapper binary.

## Module Layout
- **Entry point** (`lib.rs`): `#[cfg(unix)]` gate; re-exports all public types from the `unix` submodule
- **Unix implementation** (`unix/`): `escalate_server.rs` (server listening on Unix socket), `escalate_client.rs` (client side used by wrapper), `escalate_protocol.rs` (wire protocol and `ESCALATE_SOCKET_ENV_VAR`), `escalation_policy.rs` (`EscalationPolicy` trait), `execve_wrapper.rs` (wrapper logic with `libc` syscalls), `socket.rs` (socket pair creation), `stopwatch.rs` (pausable timer for timeouts)
- **Binary** (`bin/main_execve_wrapper.rs`): Entrypoint for the `codex-execve-wrapper` binary delegating to `main_execve_wrapper()`
