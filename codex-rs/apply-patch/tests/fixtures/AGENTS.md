# codex-rs/apply-patch/tests/fixtures/

This file applies to `codex-rs/apply-patch/tests/fixtures/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

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

Test fixture data for the apply-patch integration tests.

### What this folder does

Contains the `scenarios/` directory, which holds all end-to-end test cases for the apply-patch specification. Each scenario is a self-contained directory with input state, a patch file, and expected output state.

### What it plugs into

- Read by `tests/suite/scenarios.rs` which iterates over every scenario directory, copies input files to a temp directory, runs the `apply_patch` binary with the patch, and compares the resulting filesystem state against the expected directory.

### Key files

| File | Role |
|------|------|
| `scenarios/` | Directory of numbered test scenarios (001 through 022). |
| `scenarios/README.md` | Documents the scenario specification format. |
| `scenarios/.gitattributes` | Git attributes for fixture files. |
