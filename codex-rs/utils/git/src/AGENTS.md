# codex-rs/utils/git/src/

This file applies to `codex-rs/utils/git/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-git` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-git`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source directory for the `codex-git` crate.

### Key files

- `lib.rs` -- module declarations, `GhostCommit` struct (with `Serialize`/`Deserialize`/`JsonSchema`/`TS`), and public re-exports
- `ghost_commits.rs` -- the largest module (~900 lines):
  - `CreateGhostCommitOptions` / `RestoreGhostCommitOptions` -- builder-pattern option structs
  - `GhostSnapshotConfig` -- thresholds for ignoring large untracked files/dirs
  - `create_ghost_commit` -- uses a temporary index (`GIT_INDEX_FILE`) to snapshot working tree without disturbing user's index
  - `restore_ghost_commit` -- runs `git restore --source <commit> --worktree` then cleans new untracked files
  - `capture_status_snapshot` -- parses `git status --porcelain=2 -z` output to enumerate tracked and untracked entries
  - `should_ignore_for_snapshot` -- skips well-known dependency directories (`node_modules`, `.venv`, `build`, etc.)
  - `detect_large_untracked_dirs` -- identifies directories exceeding a file count threshold
  - Extensive tests covering roundtrip create/restore, subdirectory scoping, and preservation of ignored files
- `apply.rs` -- `apply_git_patch` function:
  - Writes diff to temp file, runs `git apply --3way` (or `--check` for preflight)
  - `parse_git_apply_output` -- regex-based parser handling ~15 different `git apply` output patterns
  - `extract_paths_from_patch` -- extracts file paths from `diff --git` headers (handles quoted/escaped paths)
  - `stage_paths` -- best-effort `git add` of existing paths
- `branch.rs` -- `merge_base_with_head`:
  - Resolves the merge-base between HEAD and a branch
  - Prefers remote upstream when it has more commits than local
- `operations.rs` -- internal git plumbing helpers:
  - `run_git` / `run_git_for_stdout` / `run_git_for_stdout_all` / `run_git_for_status` -- git command execution with error handling
  - `ensure_git_repository` -- validates path is inside a git work tree
  - `resolve_head` / `resolve_repository_root` / `repo_subdir` -- path resolution
  - `normalize_relative_path` -- validates paths stay within repository bounds
- `errors.rs` -- `GitToolingError` enum with variants for git command failures, path issues, I/O errors
- `platform.rs` -- `create_symlink` with platform-specific implementations for Unix and Windows
