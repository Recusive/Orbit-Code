# codex-rs/core/tests/fixtures/

This file applies to `codex-rs/core/tests/fixtures/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Treat this directory as golden data or generated/static support material. Keep filenames and relative paths stable unless the owning test, renderer, or generator changes first.
- Prefer updating the producer or the corresponding test scenario before editing files here by hand.
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Test fixture data files for integration tests.

### What this folder does

Contains static data files used by integration tests as inputs or expected outputs.

### Key files

| File | Purpose |
|------|---------|
| `incomplete_sse.json` | JSON fixture containing an incomplete SSE (Server-Sent Events) response, used to test error handling for truncated or malformed API responses |

### Where it plugs into

- Referenced by integration tests in `tests/suite/` via `include_str!()` or file path
- Used to verify resilience to malformed API responses
