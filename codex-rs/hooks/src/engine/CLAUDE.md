# codex-rs/hooks/src/engine/

Hook execution engine: discovery, configuration, command running, and output parsing.

## What this folder does

Implements the `ClaudeHooksEngine` that discovers configured hooks from the config layer stack, runs them as external commands with JSON input/output, and parses their responses.

## Key files and their roles

- `mod.rs` -- `ClaudeHooksEngine` struct and `ConfiguredHandler` (event_name, matcher, command, timeout_sec, status_message, source_path, display_order). The engine is initialized from a `ConfigLayerStack`: discovers handlers, loads schemas, and stores warnings. Provides `run_session_start()`, `run_user_prompt_submit()`, `run_stop()`, and corresponding `preview_*` methods. Also defines `CommandShell` (program + args).
- `discovery.rs` -- `discover_handlers()`: scans the `ConfigLayerStack` for hook definitions and returns a list of `ConfiguredHandler` entries with any warnings.
- `config.rs` -- Hook configuration types and parsing from the config layer stack.
- `command_runner.rs` -- Executes hook commands as child processes: sends JSON input on stdin, reads JSON output from stdout, enforces timeouts, and handles process failures.
- `output_parser.rs` -- Parses JSON output from hook commands into typed outcomes (continue/stop, block decisions, system messages, context additions).
- `schema_loader.rs` -- `generated_hook_schemas()`: loads the pre-generated JSON Schema fixtures at engine startup for validation.

## Imports from

- `codex-config`: ConfigLayerStack
- `codex-protocol`: HookRunSummary, HookEventName, HookRunStatus, etc.
- `crate::events::*`: per-event request/outcome types and run/preview functions
- `crate::schema`: wire format types

## Exports to

- `ClaudeHooksEngine` used by `Hooks` in `registry.rs`
- `CommandShell` used for configuring the shell that runs hook commands
