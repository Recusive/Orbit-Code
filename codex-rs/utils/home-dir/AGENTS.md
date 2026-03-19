# codex-rs/utils/home-dir/

This file applies to `codex-rs/utils/home-dir/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-utils-home-dir` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-utils-home-dir`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Crate `codex-utils-home-dir` -- locate the Codex configuration home directory.

### What this folder does

Provides the `find_codex_home()` function that resolves the Codex configuration directory. Honors the `CODEX_HOME` environment variable (which must point to an existing directory) and falls back to `~/.codex`.

### Key types and functions

- `find_codex_home() -> io::Result<PathBuf>` -- main entry point; reads `CODEX_HOME` env var, validates it exists and is a directory, canonicalizes it; falls back to `~/.codex`

### Imports from

- `dirs` -- home directory lookup

### Exports to

Used by `codex-config` and `codex-secrets` for locating configuration and secrets files.

### Key files

- `Cargo.toml` -- crate metadata; depends on `dirs`
- `src/lib.rs` -- `find_codex_home`, `find_codex_home_from_env` (testable inner function), and tests
