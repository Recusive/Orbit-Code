# codex-rs/utils/cargo-bin/

Crate `codex-utils-cargo-bin` -- locate test binaries and resources across Cargo and Bazel.

## What this folder does

Provides helpers to find compiled binary targets and test resources at runtime, transparently supporting both `cargo test` (where `CARGO_BIN_EXE_*` env vars are absolute paths) and `bazel test` (where they are rlocationpaths resolved via runfiles).

## Key types and functions

- `cargo_bin(name)` -- returns the absolute path to a binary built for the current test run
- `find_resource!` -- macro that resolves a test resource path using either Cargo's `CARGO_MANIFEST_DIR` or Bazel's runfiles
- `repo_root()` -- locate the repository root directory
- `runfiles_available()` -- check if running under Bazel
- `CargoBinError` -- error type for binary resolution failures

## Imports from

- `assert_cmd` -- fallback binary discovery via Cargo
- `runfiles` -- Bazel runfiles resolution
- `thiserror` -- error derivation

## Exports to

Used exclusively in test code across the workspace for locating test binaries and fixture files.

## Key files

- `Cargo.toml` -- crate metadata and dependencies
- `src/lib.rs` -- `cargo_bin`, `find_resource!` macro, `repo_root`, runfile helpers
