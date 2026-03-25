# codex-rs/skills/

This file applies to `codex-rs/skills/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-skills` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-skills`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Embedded system skills management for Codex.

### What this folder does

Bundles and installs built-in "system skills" (agent configurations, reference docs, scripts) into `ORBIT_HOME/skills/.system`. Skills are embedded at compile time using `include_dir` and extracted to disk on startup. A fingerprint-based marker file avoids re-extraction when the embedded content has not changed.

### What it plugs into

- Called by `codex-core` during startup to ensure system skills are available on disk.
- The skills directory structure is used by the agent skill loader.

### Imports from

- `codex-utils-absolute-path` -- path normalization.
- `include_dir` -- compile-time directory embedding.
- `thiserror` -- error type derivation.

### Exports to

- `install_system_skills(codex_home)` -- extracts embedded skills to disk.
- `system_cache_root_dir(codex_home)` -- returns the path to the system skills cache.
- `SystemSkillsError` -- error type.

### Key files

- `Cargo.toml` -- crate manifest.
- `build.rs` -- emits `cargo:rerun-if-changed` for all files under `src/assets/samples/`.
- `src/lib.rs` -- skill installation logic, fingerprinting, and disk extraction.
- `src/assets/samples/` -- embedded skill packages.
