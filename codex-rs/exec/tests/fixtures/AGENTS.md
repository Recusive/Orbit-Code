# codex-rs/exec/tests/fixtures/

This file applies to `codex-rs/exec/tests/fixtures/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Treat this directory as golden data or generated/static support material. Keep filenames and relative paths stable unless the owning test, renderer, or generator changes first.
- Prefer updating the producer or the corresponding test scenario before editing files here by hand.
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-exec` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-exec`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Test fixture data files for `codex-exec` integration tests.

### What this folder does

Stores static test data consumed by the integration test suite. These fixtures provide reproducible inputs for tests that verify CLI behavior and event processing.

### Key files and their roles

- `apply_patch_freeform_final.txt` -- Expected final output for an apply_patch freeform test case.
- `cli_responses_fixture.sse` -- Server-Sent Events (SSE) fixture used to simulate streaming CLI responses during integration tests.

### Used by

- Integration tests in `codex-rs/exec/tests/suite/` that verify end-to-end behavior of the `codex-exec` binary.
