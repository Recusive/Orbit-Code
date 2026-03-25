# codex-rs/linux-sandbox/tests/suite/

This file applies to `codex-rs/linux-sandbox/tests/suite/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-linux-sandbox` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-linux-sandbox`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Test module directory aggregated by `tests/all.rs`.

### What this folder does

Contains the actual integration test modules for the Linux sandbox. Each module exercises a different aspect of sandbox enforcement.

### Key files

| File | What it tests |
|------|---------------|
| `mod.rs` | Aggregates `landlock` and `managed_proxy` modules |
| `landlock.rs` | Comprehensive bwrap + seccomp integration tests: filesystem read/write policies, writable roots, `/dev` node availability, `.git`/`.orbit` write protection, symlink attacks, split-policy carveouts, network blocking (curl, wget, ping, nc, ssh, getent, `/dev/tcp`), `NoNewPrivs`, timeout enforcement |
| `managed_proxy.rs` | Tests for managed proxy routing mode: fail-closed behavior without proxy env vars, bridge routing through a mock proxy server, direct egress blocking, AF_UNIX socket creation denial in proxy-routed mode |

### How the tests work

Tests use `codex-core`'s `process_exec_tool_call` to invoke the `codex-linux-sandbox` binary (located via `CARGO_BIN_EXE_codex-linux-sandbox`). They construct `SandboxPolicy` / `FileSystemSandboxPolicy` objects and verify that commands either succeed or are denied. A `should_skip_bwrap_tests()` helper detects environments where bubblewrap is unavailable (no vendored build, restrictive containers) and skips gracefully.

### Imports

- `codex-core` (exec subsystem, config, sandbox permissions)
- `codex-protocol` (policy types)
- `pretty_assertions`, `tempfile`, `tokio`
