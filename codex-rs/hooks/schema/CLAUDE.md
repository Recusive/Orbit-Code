# codex-rs/hooks/schema/

JSON Schema definition files for hook input/output wire formats.

## What this folder does

Contains JSON Schema files that define the expected input (stdin) and output (stdout) formats for hook commands at each lifecycle event. These schemas serve as both documentation and validation contracts.

## Key files and their roles

- `session-start.command.input.schema.json` -- Schema for SessionStart hook input (session_id, transcript_path, cwd, model, permission_mode, source).
- `session-start.command.output.schema.json` -- Schema for SessionStart hook output (continue, stop_reason, suppress_output, system_message, hook_specific_output).
- `user-prompt-submit.command.input.schema.json` -- Schema for UserPromptSubmit hook input (session_id, turn_id, transcript_path, cwd, model, permission_mode, prompt).
- `user-prompt-submit.command.output.schema.json` -- Schema for UserPromptSubmit hook output (continue, decision, reason, hook_specific_output).
- `stop.command.input.schema.json` -- Schema for Stop hook input (session_id, turn_id, transcript_path, cwd, model, permission_mode, stop_hook_active, last_assistant_message).
- `stop.command.output.schema.json` -- Schema for Stop hook output (continue, decision, reason).

## Subfolders

- `generated/` -- Auto-generated JSON Schema files from Rust types.

## What it plugs into

- Schema files are loaded by `src/engine/schema_loader.rs` at startup
- Generated fixtures are validated in unit tests to ensure Rust types match the schema files
