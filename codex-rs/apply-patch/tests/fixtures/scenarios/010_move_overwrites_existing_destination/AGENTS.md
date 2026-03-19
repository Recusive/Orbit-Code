# codex-rs/apply-patch/tests/fixtures/scenarios/010_move_overwrites_existing_destination/

This file applies to `codex-rs/apply-patch/tests/fixtures/scenarios/010_move_overwrites_existing_destination/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Treat this directory as golden data or generated/static support material. Keep filenames and relative paths stable unless the owning test, renderer, or generator changes first.
- Prefer updating the producer or the corresponding test scenario before editing files here by hand.
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-apply-patch` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-apply-patch`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Tests that a Move To operation overwrites a file that already exists at the destination path.

### Files

- `patch.txt` -- Patch that updates and moves `old/name.txt` to `renamed/dir/name.txt`, replacing "from" with "new". The destination already has an existing file.
- `input/` -- Contains `old/name.txt`, `old/other.txt`, and a pre-existing `renamed/dir/name.txt`.
- `expected/` -- Contains `old/other.txt` (unchanged) and `renamed/dir/name.txt` (overwritten with new content).
