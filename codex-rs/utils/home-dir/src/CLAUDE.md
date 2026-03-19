# codex-rs/utils/home-dir/src/

Source directory for the `codex-utils-home-dir` crate.

## Key files

- `lib.rs` -- single-file implementation containing:
  - `find_codex_home()` -- public API that reads `CODEX_HOME` env var
  - `find_codex_home_from_env(codex_home_env: Option<&str>)` -- testable inner function that:
    - When `Some(val)`: validates path exists and is a directory, canonicalizes it
    - When `None`: returns `~/.codex` via `dirs::home_dir()`
  - Tests covering: missing path error, file-not-directory error, valid directory canonicalization, default fallback to `~/.codex`
