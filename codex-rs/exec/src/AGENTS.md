# codex-rs/exec/src/

This file applies to `codex-rs/exec/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-exec` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-exec`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source code for the `codex-exec` crate (binary + library).

### What this folder does

Contains all Rust source files that implement the headless Codex CLI. The code handles CLI argument parsing, configuration loading, in-process agent session management, and two distinct output modes (human-readable and JSONL).

### Key files and their roles

- `main.rs` -- Binary entry point. Uses `codex-arg0` for multi-binary dispatch: when invoked as `codex-linux-sandbox`, it runs sandbox logic instead of the normal exec flow. Otherwise parses `TopCli` and calls `run_main()`.
- `lib.rs` -- Core library module. Exports `Cli`, `Command`, `ReviewArgs`, `run_main()`. Contains the main event loop (`run_exec_session`), prompt resolution (stdin/arg with UTF-8/16 BOM handling), review request building, session lifecycle management, and server request handling.
- `cli.rs` -- clap-based CLI definitions. Defines `Cli` (top-level flags: model, sandbox, color, json, output-schema, etc.), `Command` enum (`Resume`, `Review`), `ResumeArgs`, and `ReviewArgs`.
- `event_processor.rs` -- Defines the `EventProcessor` trait (with `print_config_summary`, `process_event`, `print_final_output`) and `CodexStatus` enum. Also has `handle_last_message()` for writing agent's final message to a file.
- `event_processor_with_human_output.rs` -- `EventProcessorWithHumanOutput`: renders events as colorized text to stderr. Handles exec commands, patch applies, MCP tool calls, collab operations, plan updates, hooks, web search, image generation, and agent job progress bars.
- `event_processor_with_jsonl_output.rs` -- `EventProcessorWithJsonOutput`: converts the internal `Event` stream into structured `ThreadEvent` JSONL on stdout. Tracks running commands, MCP calls, collab calls, patch applies, web searches, and todo lists by call_id.
- `exec_events.rs` -- Public type definitions for the JSONL output wire format. Defines `ThreadEvent` (tagged enum), `ThreadItem`, `ThreadItemDetails` (AgentMessage, Reasoning, CommandExecution, FileChange, McpToolCall, CollabToolCall, WebSearch, TodoList, Error), and associated status/payload types.

### Imports from

- `codex-core` (Config, AuthManager, session management)
- `codex-app-server-client` (InProcessAppServerClient)
- `codex-app-server-protocol` (JSON-RPC types)
- `codex-protocol` (Event, EventMsg, all event payload types)
- `codex-arg0`, `codex-otel`, `codex-feedback`

### Exports to

- Binary entry point consumed by users/CI
- Library re-exports used by integration tests
