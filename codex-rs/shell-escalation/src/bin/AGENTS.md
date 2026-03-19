# codex-rs/shell-escalation/src/bin/

This file applies to `codex-rs/shell-escalation/src/bin/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-shell-escalation` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-shell-escalation`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Binary entrypoint for the `codex-execve-wrapper` executable.

### What this folder does

Contains the `main` function for `codex-execve-wrapper`, the binary that intercepts `exec()` calls from a patched bash shell and communicates with the escalation server.

### Key files

- `main_execve_wrapper.rs` -- on Unix, delegates to `codex_shell_escalation::main_execve_wrapper` which parses CLI args (file + argv) and runs `run_shell_escalation_execve_wrapper()`. On non-Unix platforms, prints an error and exits.

### What it plugs into

- Invoked by the patched bash shell via the `EXEC_WRAPPER` environment variable.
- Communicates with `EscalateServer` over the socket specified by `CODEX_ESCALATE_SOCKET`.
