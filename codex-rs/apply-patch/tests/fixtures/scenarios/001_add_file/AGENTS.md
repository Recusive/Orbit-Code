# codex-rs/apply-patch/tests/fixtures/scenarios/001_add_file/

This file applies to `codex-rs/apply-patch/tests/fixtures/scenarios/001_add_file/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

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

Tests that a patch with `*** Add File` creates a new file with the specified contents.

### Files

- `patch.txt` -- Patch that adds `bar.md` with content "This is a new file".
- `expected/bar.md` -- Expected output file after the patch is applied.

This scenario has no `input/` directory since no pre-existing files are needed.
