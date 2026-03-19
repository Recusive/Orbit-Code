# codex-rs/backend-client/tests/fixtures/

This file applies to `codex-rs/backend-client/tests/fixtures/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Treat this directory as golden data or generated/static support material. Keep filenames and relative paths stable unless the owning test, renderer, or generator changes first.
- Prefer updating the producer or the corresponding test scenario before editing files here by hand.
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-backend-client` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-backend-client`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

JSON fixtures for backend-client response deserialization tests.

### What this folder does

Stores realistic JSON response payloads from the Codex backend API, used by unit tests to verify that hand-rolled deserialization types and the `CodeTaskDetailsResponseExt` trait work correctly.

### Where it plugs in

- Loaded via `include_str!("../tests/fixtures/...")` in `src/types.rs` test module
- Each fixture exercises different code paths in the `CodeTaskDetailsResponse` model

### Key files

| File | Role |
|------|------|
| `task_details_with_diff.json` | Task with a user turn (two content parts), an assistant message turn, and a `current_diff_task_turn` containing an `output_diff` with a git diff string |
| `task_details_with_error.json` | Task with a failed assistant turn containing a `pr`-type output item with `output_diff.diff` and an `error` object with code `APPLY_FAILED` |
