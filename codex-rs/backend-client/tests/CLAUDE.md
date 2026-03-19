# codex-rs/backend-client/tests/

Test data for the `codex-backend-client` crate.

## What this folder does

Contains JSON fixture files used by the unit tests in `src/types.rs` to verify deserialization and the `CodeTaskDetailsResponseExt` trait methods (unified diff extraction, assistant text messages, user prompt extraction, error message parsing).

## Where it plugs in

- Fixtures are loaded via `include_str!` in the `#[cfg(test)]` module of `src/types.rs`
- Tests verify the hand-rolled response models against realistic backend JSON payloads

## Key files

| File | Role |
|------|------|
| `fixtures/` | Directory containing JSON test fixtures |
| `task_details_with_diff.json` | Symlink/copy in older layout (may duplicate `fixtures/` content) |
| `task_details_with_error.json` | Symlink/copy in older layout (may duplicate `fixtures/` content) |
