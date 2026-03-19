# codex-rs/core/src/unified_exec/

This file applies to `codex-rs/core/src/unified_exec/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Interactive process execution with PTY management, approvals, and sandboxing.

### What this folder does

The unified exec subsystem manages interactive processes (terminals) that the AI agent can create, send input to, and read output from. Unlike one-shot shell commands, these processes persist across multiple tool calls.

#### Architecture

1. Build a request (`ExecCommandRequest`) with command, cwd, permissions
2. Orchestrator handles: approval (bypass/cache/prompt) -> select sandbox -> run
3. Runtime transforms `CommandSpec` -> `ExecRequest` -> spawn PTY
4. If sandbox denial, orchestrator retries with `SandboxType::None`
5. Process handle returned with streaming output + metadata

#### Key features
- **Process pooling**: Up to 64 concurrent processes per session (warning at 60)
- **Output buffering**: Head/tail buffer with 1 MiB max output, ~256K token limit
- **Yield timing**: Configurable wait times (250ms - 30s) for output collection
- **Background processes**: Up to 5 minutes max timeout for background terminals
- **Process reuse**: Processes can be written to multiple times via `write_stdin`
- **Deterministic IDs**: Test mode for reproducible process IDs

### Key files

| File | Purpose |
|------|---------|
| `mod.rs` | `UnifiedExecProcessManager`, `ExecCommandRequest`, `WriteStdinRequest`, constants, process store |
| `process.rs` | `UnifiedExecProcess` -- PTY process lifecycle, output buffering, spawn lifecycle hooks |
| `process_manager.rs` | Orchestration: approvals, sandboxing, process reuse, request handling |
| `head_tail_buffer.rs` | `HeadTailBuffer` -- keeps first and last N bytes of output for large outputs |
| `async_watcher.rs` | Async output watching and notification |
| `errors.rs` | `UnifiedExecError` -- specialized error types |

### Imports from

- `crate::codex` -- `Session`, `TurnContext`
- `crate::sandboxing` -- `SandboxPermissions`, command transformation
- `codex_network_proxy` -- `NetworkProxy` for managed network access
- `codex_protocol` -- `PermissionProfile`

### Exports to

- `crate::tools::handlers::unified_exec` -- `UnifiedExecHandler` uses the process manager
- `crate::state::SessionServices` -- `UnifiedExecProcessManager` held in session services
- `crate::tasks` -- cleanup of unified exec processes on session teardown
