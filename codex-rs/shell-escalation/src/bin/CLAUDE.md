# codex-rs/shell-escalation/src/bin/

Binary entrypoint for the `codex-execve-wrapper` executable.

## What this folder does

Contains the `main` function for `codex-execve-wrapper`, the binary that intercepts `exec()` calls from a patched bash shell and communicates with the escalation server.

## Key files

- `main_execve_wrapper.rs` -- on Unix, delegates to `codex_shell_escalation::main_execve_wrapper` which parses CLI args (file + argv) and runs `run_shell_escalation_execve_wrapper()`. On non-Unix platforms, prints an error and exits.

## What it plugs into

- Invoked by the patched bash shell via the `EXEC_WRAPPER` environment variable.
- Communicates with `EscalateServer` over the socket specified by `CODEX_ESCALATE_SOCKET`.
