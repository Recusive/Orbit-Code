# codex-rs/skills/src/assets/

This file applies to `codex-rs/skills/src/assets/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Treat this directory as golden data or generated/static support material. Keep filenames and relative paths stable unless the owning test, renderer, or generator changes first.
- Prefer updating the producer or the corresponding test scenario before editing files here by hand.
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-skills` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-skills`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Static assets embedded into the `codex-skills` binary at compile time.

### What this folder does

Contains the `samples/` directory with built-in skill packages that are embedded via `include_dir!()` and extracted to `ORBIT_HOME/skills/.system` on startup.

### Key subdirectories

- `samples/` -- individual skill packages (openai-docs, skill-creator, skill-installer).
