# codex-rs/utils/cargo-bin/src/

This file applies to `codex-rs/utils/cargo-bin/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-utils-cargo-bin` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-utils-cargo-bin`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source directory for the `codex-utils-cargo-bin` crate.

### Key files

- `lib.rs` -- single-file implementation containing:
  - `cargo_bin(name: &str) -> Result<PathBuf, CargoBinError>` -- tries `CARGO_BIN_EXE_*` env vars first (handling both Cargo absolute paths and Bazel rlocationpaths), then falls back to `assert_cmd::Command::cargo_bin`
  - `find_resource!` macro -- compile-time macro that reads `CARGO_MANIFEST_DIR` or `BAZEL_PACKAGE` to resolve test resource paths at runtime
  - `resolve_bazel_runfile` / `resolve_cargo_runfile` -- internal helpers for each build system
  - `repo_root()` -- walks up from a `repo_root.marker` file to find the repository root
  - `normalize_runfile_path` -- collapses `.` and `..` components in runfile paths
  - `CargoBinError` enum -- variants for missing exe, resolution failures, and path-not-found
