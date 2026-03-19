# codex-rs/skills/src/

Source code for the `codex-skills` crate.

## What this folder does

Contains the library implementation and the embedded skill assets.

## Key files

- `lib.rs` -- the entire library implementation. Key functions: `install_system_skills()` extracts embedded skills to `CODEX_HOME/skills/.system` with fingerprint-based caching; `system_cache_root_dir()` returns the cache path; `embedded_system_skills_fingerprint()` computes a hash of all embedded files for change detection.
- `assets/` -- contains the embedded skill samples that are compiled into the binary.

## Exports to

- Parent crate re-exports all public items from `lib.rs`.
