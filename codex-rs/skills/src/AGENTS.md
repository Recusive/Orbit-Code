# codex-rs/skills/src/

This file applies to `codex-rs/skills/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-skills` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-skills`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source code for the `codex-skills` crate.

### What this folder does

Contains the library implementation and the embedded skill assets.

### Key files

- `lib.rs` -- the entire library implementation. Key functions: `install_system_skills()` extracts embedded skills to `ORBIT_HOME/skills/.system` with fingerprint-based caching; `system_cache_root_dir()` returns the cache path; `embedded_system_skills_fingerprint()` computes a hash of all embedded files for change detection.
- `assets/` -- contains the embedded skill samples that are compiled into the binary.

### Exports to

- Parent crate re-exports all public items from `lib.rs`.
