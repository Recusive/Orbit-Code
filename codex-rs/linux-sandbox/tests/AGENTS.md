# codex-rs/linux-sandbox/tests/

This file applies to `codex-rs/linux-sandbox/tests/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-linux-sandbox` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-linux-sandbox`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Integration tests for the `codex-linux-sandbox` crate. All tests are Linux-only (`#[cfg(target_os = "linux")]`).

### What this folder does

Validates end-to-end sandbox behavior by running real commands through the `codex-linux-sandbox` binary (via `codex-core`'s `process_exec_tool_call`) and asserting on filesystem access, network blocking, and proxy routing.

### Structure

- `all.rs` -- Single integration test binary entry point; pulls in the `suite` module.
- `suite/` -- Contains the actual test modules.

### What it tests

The tests verify:
- Filesystem read/write restrictions (bubblewrap)
- `/dev/null` writeability and minimal device node availability
- Writable root enforcement, including missing roots
- `.git` and `.codex` subpath protection (including symlink replacement attacks)
- Explicit split-policy carveouts under bwrap
- Network blocking (curl, wget, ping, nc, ssh, getent, `/dev/tcp`)
- `NoNewPrivs` is active in sandboxed processes
- Timeout enforcement
- Managed proxy routing bridges

### Dependencies

- `codex-core` (exec subsystem, config types)
- `codex-protocol` (sandbox policy types)
- `pretty_assertions`, `tempfile`, `tokio`
