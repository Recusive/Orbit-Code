# codex-rs/utils/cargo-bin/src/

Source directory for the `codex-utils-cargo-bin` crate.

## Key files

- `lib.rs` -- single-file implementation containing:
  - `cargo_bin(name: &str) -> Result<PathBuf, CargoBinError>` -- tries `CARGO_BIN_EXE_*` env vars first (handling both Cargo absolute paths and Bazel rlocationpaths), then falls back to `assert_cmd::Command::cargo_bin`
  - `find_resource!` macro -- compile-time macro that reads `CARGO_MANIFEST_DIR` or `BAZEL_PACKAGE` to resolve test resource paths at runtime
  - `resolve_bazel_runfile` / `resolve_cargo_runfile` -- internal helpers for each build system
  - `repo_root()` -- walks up from a `repo_root.marker` file to find the repository root
  - `normalize_runfile_path` -- collapses `.` and `..` components in runfile paths
  - `CargoBinError` enum -- variants for missing exe, resolution failures, and path-not-found
