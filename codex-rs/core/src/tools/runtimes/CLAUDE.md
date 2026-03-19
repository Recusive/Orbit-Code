# codex-rs/core/src/tools/runtimes/

Concrete `ToolRuntime` implementations for specific execution backends.

## What this folder does

Each runtime stays small and focused, reusing the `ToolOrchestrator` for approvals + sandbox + retry logic. Runtimes transform tool-specific requests into `CommandSpec` instances that the sandbox manager can process.

### Available runtimes

- **Shell** (`shell/`): Executes shell commands via `bash -lc` or `zsh -lc` with optional shell snapshot sourcing.
- **Apply patch** (`apply_patch.rs`): Runs the `apply_patch` binary to apply file diffs.
- **Unified exec** (`unified_exec.rs`): Creates and manages interactive PTY processes.

### Shared utilities (`mod.rs`)
- `build_command_spec()`: Constructs a `CommandSpec` from a tokenized command line.
- `maybe_wrap_shell_lc_with_snapshot()`: POSIX-only helper that rewrites `shell -lc "<script>"` to source a shell snapshot before execution, preserving environment variables across snapshot loads.
- `ExecveSessionApproval`: Tracks skill metadata for execve-based session approvals.

## Key files

| File | Purpose |
|------|---------|
| `mod.rs` | `build_command_spec()`, `maybe_wrap_shell_lc_with_snapshot()`, `ExecveSessionApproval` |
| `shell/` | Shell command execution runtime |
| `apply_patch.rs` | Apply-patch execution runtime |
| `unified_exec.rs` | Unified exec (interactive PTY) runtime |

## Imports from

- `crate::sandboxing` -- `CommandSpec`, `SandboxPermissions`
- `crate::tools::sandboxing` -- `ToolError`, `ToolCtx`, `SandboxAttempt`
- `crate::exec` -- `ExecExpiration`
- `crate::shell` -- `Shell` for shell detection
- `crate::skills` -- `SkillMetadata` for skill-backed executions

## Exports to

- `crate::tools::orchestrator` -- Runtimes are called by the orchestrator during tool execution
- `crate::tools::handlers` -- Handlers construct runtime-specific requests
