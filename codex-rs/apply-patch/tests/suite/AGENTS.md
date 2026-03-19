# codex-rs/apply-patch/tests/suite/

This file applies to `codex-rs/apply-patch/tests/suite/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-apply-patch` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-apply-patch`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Test module directory containing the integration test implementations for the `apply_patch` binary.

### What this folder does

Provides three test modules that exercise the `apply_patch` binary through subprocess invocations, covering CLI argument handling, stdin input, multi-operation patches, error reporting, and scenario-based filesystem verification.

### What it plugs into

- Aggregated by `tests/all.rs` via `mod suite`.
- Uses `codex-utils-cargo-bin` to locate the compiled binary.
- Uses `assert_cmd` for CLI assertions and `tempfile` for isolated directories.

### Key files

| File | Role |
|------|------|
| `mod.rs` | Module declaration; imports `cli`, `scenarios`, and `tool` (Unix-only). |
| `cli.rs` | CLI tests using `assert_cmd`. Tests stdin/argument input, add+update sequences, and stdout/stderr output validation. |
| `scenarios.rs` | Scenario runner. Iterates over `tests/fixtures/scenarios/`, copies input to temp dirs, runs the binary, and compares resulting filesystem against expected state using `BTreeMap` snapshots. |
| `tool.rs` | Additional CLI tool tests (Unix-only, gated by `#[cfg(not(target_os = "windows"))]`). Tests multi-operation patches, multiple chunks, file moves, error cases (empty patch, missing context, missing file, invalid headers, directory delete), partial failure semantics, and trailing newline behavior. |
