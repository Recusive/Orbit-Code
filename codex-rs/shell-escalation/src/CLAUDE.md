# codex-rs/shell-escalation/src/

Source code for the shell-escalation crate.

## What this folder does

Contains the library entry point and the Unix-specific implementation module.

## Key files

- `lib.rs` -- conditional compilation gate: all types are only available on `cfg(unix)`. Re-exports the public API from the `unix` submodule.
- `unix/` -- the full Unix implementation of the escalation protocol.
- `bin/main_execve_wrapper.rs` -- binary entrypoint that delegates to `main_execve_wrapper()`.

## Exports to

- The parent crate (`codex-shell-escalation`) re-exports everything defined here.
