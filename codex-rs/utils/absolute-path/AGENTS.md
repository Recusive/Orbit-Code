# codex-rs/utils/absolute-path/

This file applies to `codex-rs/utils/absolute-path/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-utils-absolute-path` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-utils-absolute-path`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Crate `codex-utils-absolute-path` -- a newtype wrapper guaranteeing paths are absolute and normalized.

### What this folder does

Provides `AbsolutePathBuf`, a path type that is always absolute and normalized (though not necessarily canonicalized or existing on disk). Supports tilde (`~`) expansion on non-Windows platforms, relative path resolution against a base, and serde deserialization with a thread-local base path guard.

### Key types and functions

- `AbsolutePathBuf` -- the core newtype wrapping `PathBuf`; implements `Serialize`, `Deserialize`, `JsonSchema`, and `TS`
- `AbsolutePathBufGuard` -- RAII guard that sets a thread-local base path for deserializing relative paths
- `resolve_path_against_base()` -- resolve a possibly-relative path against an explicit base
- `from_absolute_path()` -- construct from an already-absolute path
- `current_dir()` -- construct from the current working directory

### Imports from

- `dirs` -- home directory lookup for tilde expansion
- `path-absolutize` -- path normalization
- `schemars`, `serde`, `ts-rs` -- schema generation and serialization

### Exports to

Used extensively throughout the workspace wherever absolute paths are required in configuration, sandbox policies, and protocol types. Key consumers include `codex-protocol`, `codex-config`, `codex-core`, and `codex-utils-sandbox-summary`.

### Key files

- `Cargo.toml` -- crate metadata and dependencies
- `src/lib.rs` -- all implementation: `AbsolutePathBuf`, `AbsolutePathBufGuard`, trait impls, tests
