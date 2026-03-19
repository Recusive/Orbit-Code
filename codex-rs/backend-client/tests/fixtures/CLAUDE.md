# codex-rs/backend-client/tests/fixtures/

JSON fixtures for backend-client response deserialization tests.

## What this folder does

Stores realistic JSON response payloads from the Codex backend API, used by unit tests to verify that hand-rolled deserialization types and the `CodeTaskDetailsResponseExt` trait work correctly.

## Where it plugs in

- Loaded via `include_str!("../tests/fixtures/...")` in `src/types.rs` test module
- Each fixture exercises different code paths in the `CodeTaskDetailsResponse` model

## Key files

| File | Role |
|------|------|
| `task_details_with_diff.json` | Task with a user turn (two content parts), an assistant message turn, and a `current_diff_task_turn` containing an `output_diff` with a git diff string |
| `task_details_with_error.json` | Task with a failed assistant turn containing a `pr`-type output item with `output_diff.diff` and an `error` object with code `APPLY_FAILED` |
