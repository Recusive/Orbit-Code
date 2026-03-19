# codex-rs/utils/home-dir/

Crate `codex-utils-home-dir` -- locate the Codex configuration home directory.

## What this folder does

Provides the `find_codex_home()` function that resolves the Codex configuration directory. Honors the `CODEX_HOME` environment variable (which must point to an existing directory) and falls back to `~/.codex`.

## Key types and functions

- `find_codex_home() -> io::Result<PathBuf>` -- main entry point; reads `CODEX_HOME` env var, validates it exists and is a directory, canonicalizes it; falls back to `~/.codex`

## Imports from

- `dirs` -- home directory lookup

## Exports to

Used by `codex-config` and `codex-secrets` for locating configuration and secrets files.

## Key files

- `Cargo.toml` -- crate metadata; depends on `dirs`
- `src/lib.rs` -- `find_codex_home`, `find_codex_home_from_env` (testable inner function), and tests
