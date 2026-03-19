# codex-rs/apply-patch/tests/fixtures/scenarios/

This file applies to `codex-rs/apply-patch/tests/fixtures/scenarios/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Treat this directory as golden data or generated/static support material. Keep filenames and relative paths stable unless the owning test, renderer, or generator changes first.
- Prefer updating the producer or the corresponding test scenario before editing files here by hand.
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-apply-patch` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-apply-patch`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Collection of end-to-end test scenarios for the apply-patch specification, designed to be portable across languages and platforms.

### What this folder does

Each numbered subdirectory is a self-contained test case. Every scenario consists of:
- `input/` -- initial filesystem state (copied to a temp directory before running)
- `patch.txt` -- the patch to apply
- `expected/` -- the expected filesystem state after applying the patch

The test runner (`tests/suite/scenarios.rs`) iterates over all directories here, applies the patch, and asserts the final filesystem state matches `expected/` exactly.

### What it plugs into

- Consumed by `tests/suite/scenarios.rs` via the `test_apply_patch_scenarios()` test function.
- The `apply_patch` binary is invoked as a subprocess within each scenario.

### Scenarios

| Directory | Tests |
|-----------|-------|
| `001_add_file` | Adding a new file |
| `002_multiple_operations` | Add, delete, and update in a single patch |
| `003_multiple_chunks` | Multiple update chunks within one file |
| `004_move_to_new_directory` | Move/rename a file to a new directory path |
| `005_rejects_empty_patch` | Empty patch (no hunks) is rejected; input unchanged |
| `006_rejects_missing_context` | Update with non-matching context lines fails; input unchanged |
| `007_rejects_missing_file_delete` | Deleting a non-existent file fails; input unchanged |
| `008_rejects_empty_update_hunk` | Update hunk with no diff lines is rejected |
| `009_requires_existing_file_for_update` | Updating a non-existent file fails |
| `010_move_overwrites_existing_destination` | Move overwrites a file already at the destination |
| `011_add_overwrites_existing_file` | Add File overwrites an existing file |
| `012_delete_directory_fails` | Deleting a directory (not a file) fails |
| `013_rejects_invalid_hunk_header` | Invalid hunk header syntax is rejected |
| `014_update_file_appends_trailing_newline` | Updated file gets a trailing newline appended |
| `015_failure_after_partial_success_leaves_changes` | Earlier successful hunks persist even if a later hunk fails |
| `016_pure_addition_update_chunk` | Update chunk with only additions (no old lines) |
| `017_whitespace_padded_hunk_header` | Hunk header with leading whitespace is tolerated |
| `018_whitespace_padded_patch_markers` | Begin/End Patch markers with extra whitespace are tolerated |
| `019_unicode_simple` | Patch with Unicode characters (accented, emoji) |
| `020_delete_file_success` | Successfully deleting an existing file |
| `020_whitespace_padded_patch_marker_lines` | Patch markers with trailing whitespace on marker lines |
| `021_update_file_deletion_only` | Update that only removes lines (no additions) |
| `022_update_file_end_of_file_marker` | Update using `*** End of File` marker for EOF-anchored changes |
