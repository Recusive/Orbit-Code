# codex-rs/core/src/sandboxing/

This file applies to `codex-rs/core/src/sandboxing/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Platform sandbox wrappers and command transformation for secure execution.

### What this folder does

Owns the low-level sandbox placement and transformation of portable `CommandSpec` into ready-to-spawn `ExecRequest`. The `SandboxManager` is the central coordinator that:

1. **Selects sandbox type**: Based on filesystem/network policies, platform capabilities, and managed requirements, decides whether to use macOS Seatbelt, Linux seccomp/landlock, Windows restricted tokens, or no sandbox.
2. **Transforms commands**: Wraps the original command with sandbox enforcement (e.g., prepending `sandbox-exec` on macOS, `codex-linux-sandbox` on Linux).
3. **Manages permissions**: Merges additional permission profiles (file system read/write paths, network access, macOS seatbelt extensions) with the base sandbox policy.
4. **Detects denials**: `denied()` method checks if command output indicates a sandbox denial.
5. **Executes**: `execute_env()` runs the transformed `ExecRequest`.

### Key files

| File | Purpose |
|------|---------|
| `mod.rs` | `SandboxManager`, `CommandSpec`, `ExecRequest`, `SandboxTransformRequest`, permission merging/intersection logic, `execute_env()` |
| `macos_permissions.rs` | macOS-specific seatbelt profile extension merging and intersection |
| `macos_permissions_tests.rs` | Tests for macOS permission operations |
| `mod_tests.rs` | Tests for sandbox transformation and permission merging |

### Key types

- `CommandSpec`: Portable command description (program, args, cwd, env, permissions)
- `ExecRequest`: Fully resolved command ready for spawning (with sandbox wrapper, policies, env vars)
- `SandboxTransformRequest`: Bundled arguments for sandbox transformation
- `EffectiveSandboxPermissions`: Combined base + additional permissions
- `SandboxPreference`: Auto / Require / Forbid

### Imports from

- `crate::exec` -- `ExecExpiration`, `ExecToolCallOutput`, `SandboxType`, `execute_exec_request`
- `crate::seatbelt` -- macOS Seatbelt profile generation
- `crate::landlock` -- Linux landlock/seccomp sandbox argument construction
- `codex_protocol` -- `SandboxPolicy`, `FileSystemSandboxPolicy`, `NetworkSandboxPolicy`, `PermissionProfile`
- `codex_network_proxy` -- `NetworkProxy` for managed network enforcement

### Exports to

- `crate::tools::sandboxing` -- `SandboxManager` used by `ToolRuntime` implementations
- `crate::tools::orchestrator` -- sandbox selection and retry logic
- `crate::tools::runtimes` -- `CommandSpec` construction helpers
- `crate::unified_exec` -- sandbox transformation for interactive processes
