# codex-rs/exec/tests/

Integration and unit tests for the `codex-exec` crate.

## What this folder does

Contains integration tests for the headless Codex CLI binary and unit tests for the JSONL event processor. Tests are organized as a single binary (`all.rs`) that aggregates submodules.

## Key files and their roles

- `all.rs` -- Test binary entry point. Imports `suite` (integration tests) and `event_processor_with_json_output` (JSONL processor unit tests).
- `event_processor_with_json_output.rs` -- Comprehensive unit tests for `EventProcessorWithJsonOutput`. Tests event-to-JSONL conversion for: session configured, agent messages, reasoning, command execution (begin/end, success/failure, output deltas), MCP tool calls (success/failure, null arguments, structured content), collab tool calls (spawn, interaction, wait, close), patch apply (success/failure), web search, plan/todo updates, warnings, errors, stream errors, and turn completion with usage.
- `suite/` -- Integration test modules (subcommand tests).
- `fixtures/` -- Test fixture data files.

## Imports from

- `codex_exec` (library crate: `EventProcessorWithJsonOutput`, `exec_events` types)
- `codex_protocol` (Event, EventMsg, all event payload types)
- `core_test_support` (test infrastructure from codex-core)

## What it tests

- Correctness of the JSONL event stream format (ThreadEvent wire shape)
- Proper item lifecycle tracking (started -> updated -> completed)
- Edge cases: missing begin events, out-of-order events, orphan end events
- Integration: codex-exec binary behavior with various CLI flags (add-dir, apply-patch, auth, ephemeral, MCP, originator, output-schema, resume, sandbox, server errors)
