# codex-rs/core/src/tools/code_mode/

This file applies to `codex-rs/core/src/tools/code_mode/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Interactive code execution mode with persistent background processes.

### What this folder does

Implements the "code mode" tool that allows the AI agent to execute code interactively through a persistent background process (typically a Node.js runtime). Unlike one-shot shell commands, code mode maintains state between executions.

Key components:
- **Service** (`service.rs`): `CodeModeService` -- manages the lifecycle of the background code execution process.
- **Process** (`process.rs`): Spawns and manages the Node.js child process.
- **Protocol** (`protocol.rs`): Communication protocol between Codex and the code runner (JSON messages over stdin/stdout).
- **Execute handler** (`execute_handler.rs`): `CodeModeExecuteHandler` -- handles `code_mode_execute` tool calls.
- **Wait handler** (`wait_handler.rs`): `CodeModeWaitHandler` -- handles `code_mode_wait` tool calls for long-running executions.
- **Worker** (`worker.rs`): Background worker that processes code execution requests.
- **Bridge** (`bridge.js`): JavaScript bridge script that runs inside the Node.js process.
- **Runner** (`runner.cjs`): CommonJS runner script for code execution.

### Key files

| File | Purpose |
|------|---------|
| `mod.rs` | Module declarations, tool spec augmentation, tool registration |
| `execute_handler.rs` | `CodeModeExecuteHandler` -- processes code execution tool calls |
| `wait_handler.rs` | `CodeModeWaitHandler` -- waits for long-running code results |
| `service.rs` | `CodeModeService` -- process lifecycle management |
| `process.rs` | Node.js child process spawning and management |
| `protocol.rs` | JSON-based communication protocol |
| `worker.rs` | Background execution worker |
| `bridge.js` | JavaScript bridge for the code runner |
| `runner.cjs` | CommonJS code execution runner |
| `description.md` | Tool description template for the execute tool |
| `wait_description.md` | Tool description template for the wait tool |

### Imports from

- `crate::tools` -- `ToolRouter`, `ToolCallRuntime`, `ToolPayload`, `FunctionToolOutput`
- `crate::codex` -- `Session`, `TurnContext`
- `crate::truncate` -- Output truncation

### Exports to

- `crate::tools::handlers` -- `CodeModeExecuteHandler`, `CodeModeWaitHandler`
- `crate::state::SessionServices` -- `CodeModeService`
