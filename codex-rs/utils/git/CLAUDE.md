# codex-rs/utils/git/

Git operations for undo/redo (ghost commits), applying/reverting unified diffs via `git apply --3way`, computing merge bases, and cross-platform symlink creation.

## Build & Test
```bash
cargo build -p orbit-code-git
cargo test -p orbit-code-git
```

## Key Considerations
- Ghost commits use a temporary `GIT_INDEX_FILE` to snapshot the working tree without disturbing the user's index or branch history.
- `GhostSnapshotConfig` controls thresholds for excluding large untracked files/directories from snapshots.
- `ghost_commits.rs` is ~900 lines -- new ghost commit functionality should go in a separate module if possible.
