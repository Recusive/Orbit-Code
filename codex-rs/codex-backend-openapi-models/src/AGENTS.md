# codex-rs/codex-backend-openapi-models/src/

This file applies to `codex-rs/codex-backend-openapi-models/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-backend-openapi-models` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-backend-openapi-models`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source directory for the `codex-backend-openapi-models` crate.

### What this folder does

Contains the generated OpenAPI model types. `lib.rs` simply re-exports the `models` submodule.

### Key files

| File | Role |
|------|------|
| `lib.rs` | Suppresses lint warnings; re-exports `models` module |
| `models/mod.rs` | Generated module that re-exports all individual model files |
