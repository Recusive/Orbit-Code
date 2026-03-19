# codex-rs/skills/

Embedded system skills management for Codex.

## What this folder does

Bundles and installs built-in "system skills" (agent configurations, reference docs, scripts) into `CODEX_HOME/skills/.system`. Skills are embedded at compile time using `include_dir` and extracted to disk on startup. A fingerprint-based marker file avoids re-extraction when the embedded content has not changed.

## What it plugs into

- Called by `codex-core` during startup to ensure system skills are available on disk.
- The skills directory structure is used by the agent skill loader.

## Imports from

- `codex-utils-absolute-path` -- path normalization.
- `include_dir` -- compile-time directory embedding.
- `thiserror` -- error type derivation.

## Exports to

- `install_system_skills(codex_home)` -- extracts embedded skills to disk.
- `system_cache_root_dir(codex_home)` -- returns the path to the system skills cache.
- `SystemSkillsError` -- error type.

## Key files

- `Cargo.toml` -- crate manifest.
- `build.rs` -- emits `cargo:rerun-if-changed` for all files under `src/assets/samples/`.
- `src/lib.rs` -- skill installation logic, fingerprinting, and disk extraction.
- `src/assets/samples/` -- embedded skill packages.
