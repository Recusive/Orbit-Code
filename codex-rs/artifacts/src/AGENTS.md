# codex-rs/artifacts/src/

This file applies to `codex-rs/artifacts/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-artifacts` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-artifacts`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source directory for the `codex-artifacts` crate.

### What this folder does

Contains the implementation of the artifact runtime management and build execution system. The code is organized into a top-level client module and a `runtime/` submodule tree.

### Where it plugs in

- `lib.rs` is the crate entry point, re-exporting all public types from `client.rs` and `runtime/`
- The `runtime/` subdirectory handles runtime discovery, installation, and validation
- `client.rs` consumes the runtime layer to execute artifact build commands

### Imports from

- `crate::runtime::*` -- all runtime types (manager, installed runtime, JS runtime, errors)
- `codex-package-manager` -- package download and cache management
- `reqwest`, `tokio`, `serde_json`, `tempfile`, `url`, `which` -- external dependencies

### Exports to

All public types are re-exported through `lib.rs` to crate consumers. See the parent `CLAUDE.md` for the full public API list.

### Key files

| File | Role |
|------|------|
| `lib.rs` | Crate root; declares `client`, `runtime`, and `tests` modules; re-exports public API |
| `client.rs` | `ArtifactsClient` -- wraps a runtime source (managed or pre-installed), spawns JS build commands with timeout enforcement, captures output |
| `tests.rs` | Crate-level integration tests (compiled only on non-Windows) |
| `runtime/` | Submodule for runtime discovery, download, validation, and JS executable resolution |
