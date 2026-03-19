# codex-rs/hooks/src/

Source code for the `codex-hooks` crate.

## What this folder does

Implements the hook system: configuration discovery, command execution, output parsing, and lifecycle event dispatching.

## Key files and their roles

- `lib.rs` -- Module declarations and public re-exports for all hook types.
- `registry.rs` -- `Hooks` struct: main API. `new(config)` builds the engine from `HooksConfig`. `dispatch(payload)` runs legacy hooks. `run_session_start()`, `run_user_prompt_submit()`, `run_stop()` run the engine hooks. `preview_*` methods return `HookRunSummary` without executing. Also has `command_from_argv()` utility.
- `types.rs` -- Core types: `Hook` (name + async function), `HookPayload` (session_id, cwd, client, triggered_at, hook_event), `HookEvent` (AfterAgent/AfterToolUse), `HookResult` (Success/FailedContinue/FailedAbort), `HookResponse`, `HookToolInput`, `HookToolInputLocalShell`, `HookToolKind`.
- `schema.rs` -- JSON Schema wire types and generation. Defines input/output structs for each event type (SessionStartCommandInput, UserPromptSubmitCommandInput, StopCommandInput, *OutputWire). Has `write_schema_fixtures()` for generating schema JSON files. Also defines `NullableString`, `BlockDecisionWire`, `HookEventNameWire`.
- `legacy_notify.rs` -- `legacy_notify_json()` and `notify_hook()`: backward-compatible fire-and-forget notification hooks.
- `user_notification.rs` -- `UserNotification` enum and serialization for legacy notify payloads (agent-turn-complete).

## Subfolders

- `bin/` -- Binary for generating schema fixtures
- `engine/` -- Hook discovery, configuration, command execution, and output parsing
- `events/` -- Per-event-type request/outcome types and run/preview logic
