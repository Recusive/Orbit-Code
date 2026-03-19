# codex-rs/utils/git/

This file applies to `codex-rs/utils/git/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-git` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-git`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Crate `codex-git` -- Git operations for snapshot management, patch application, and branch analysis.

### What this folder does

Provides the Git tooling layer used by Codex for undo/redo functionality (ghost commits), applying/reverting unified diffs, computing merge bases, and creating cross-platform symlinks. Ghost commits are detached commit objects that snapshot the full working tree state without modifying branch history, enabling safe undo of agent actions.

### Key types and functions

- `GhostCommit` -- serializable record of a snapshot commit with parent and preserved untracked file lists
- `create_ghost_commit` / `create_ghost_commit_with_report` -- capture the working tree into a detached commit using plumbing commands (`read-tree`, `add`, `write-tree`, `commit-tree`)
- `restore_ghost_commit` / `restore_ghost_commit_with_options` -- restore working tree to a snapshot state, cleaning up files created after the snapshot
- `capture_ghost_snapshot_report` -- generate a report without creating a commit
- `apply_git_patch` / `ApplyGitRequest` / `ApplyGitResult` -- apply unified diffs via `git apply --3way` with structured output parsing
- `extract_paths_from_patch` / `parse_git_apply_output` -- parse diff headers and `git apply` output into applied/skipped/conflicted paths
- `merge_base_with_head` -- compute the merge-base between HEAD and a branch, preferring the remote upstream when it is ahead
- `create_symlink` -- cross-platform symlink creation (Unix/Windows)
- `GitToolingError` -- comprehensive error type for all git operations
- `GhostSnapshotConfig` -- configurable thresholds for excluding large untracked files/directories from snapshots

### Imports from

- `once_cell`, `regex` -- lazy static regex patterns for output parsing
- `schemars`, `serde`, `ts-rs` -- serialization and schema generation for `GhostCommit`
- `tempfile` -- temporary files for patch application
- `thiserror` -- error derivation
- `walkdir` -- directory traversal (used in tests)

### Exports to

Consumed by `codex-core` for undo/redo, patch application, and git-aware session management.

### Key files

- `Cargo.toml` -- crate metadata and dependencies
- `src/lib.rs` -- module declarations, `GhostCommit` struct, re-exports
- `src/ghost_commits.rs` -- ghost commit create/restore logic with configurable snapshot options
- `src/apply.rs` -- `git apply` wrapper with output parsing (ported from VS Code TypeScript)
- `src/branch.rs` -- `merge_base_with_head` with upstream-aware resolution
- `src/operations.rs` -- low-level git plumbing helpers: `run_git_for_stdout`, `ensure_git_repository`, `resolve_head`, path normalization
- `src/errors.rs` -- `GitToolingError` enum
- `src/platform.rs` -- cross-platform `create_symlink`
