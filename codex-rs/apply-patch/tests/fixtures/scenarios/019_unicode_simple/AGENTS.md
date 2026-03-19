# codex-rs/apply-patch/tests/fixtures/scenarios/019_unicode_simple/

This file applies to `codex-rs/apply-patch/tests/fixtures/scenarios/019_unicode_simple/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

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

Tests that patches containing Unicode characters (accented characters, emoji) are applied correctly.

### Files

- `patch.txt` -- Patch that updates `foo.txt`, replacing a line with accented characters and adding an emoji.
- `input/foo.txt` -- Original file with Unicode content including accented characters.
- `expected/foo.txt` -- Updated file with the modified Unicode line.
