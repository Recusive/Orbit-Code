# codex-rs/utils/home-dir/src/

This file applies to `codex-rs/utils/home-dir/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-utils-home-dir` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-utils-home-dir`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source directory for the `codex-utils-home-dir` crate.

### Key files

- `lib.rs` -- single-file implementation containing:
  - `find_codex_home()` -- public API that reads `ORBIT_HOME` env var
  - `find_codex_home_from_env(codex_home_env: Option<&str>)` -- testable inner function that:
    - When `Some(val)`: validates path exists and is a directory, canonicalizes it
    - When `None`: returns `~/.orbit` via `dirs::home_dir()`
  - Tests covering: missing path error, file-not-directory error, valid directory canonicalization, default fallback to `~/.orbit`
