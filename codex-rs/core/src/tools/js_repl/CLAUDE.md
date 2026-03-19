# codex-rs/core/src/tools/js_repl/

JavaScript REPL tool with persistent kernel and code analysis.

## What this folder does

Provides a JavaScript REPL (Read-Eval-Print Loop) tool that the AI agent can use to execute JavaScript code interactively. Unlike shell commands, the REPL maintains state between executions within a turn.

Key components:
- **Kernel management** (`mod.rs`): Manages the Node.js REPL kernel lifecycle -- spawning, restarting, and per-turn isolation.
- **Code execution**: Sends JavaScript code to the kernel, captures output (including images via base64), and returns results.
- **Parser** (`meriyah.umd.min.js`): Bundled JavaScript parser for static analysis of code snippets.
- **Kernel script** (`kernel.js`): The Node.js REPL kernel that executes code and communicates results.

## Key files

| File | Purpose |
|------|---------|
| `mod.rs` | REPL manager, kernel lifecycle, tool spec definitions |
| `mod_tests.rs` | Tests for REPL execution |
| `kernel.js` | Node.js REPL kernel script |
| `meriyah.umd.min.js` | Bundled JavaScript parser (UMD format) |

## Imports from

- `crate::codex` -- `Session`, `TurnContext`
- `crate::tools::context` -- `ToolPayload`, `FunctionToolOutput`
- `crate::exec` -- Execution infrastructure

## Exports to

- `crate::tools::handlers::js_repl` -- `JsReplHandler`, `JsReplResetHandler` handlers
- `crate::state::SessionServices` -- REPL state persisted across turns
