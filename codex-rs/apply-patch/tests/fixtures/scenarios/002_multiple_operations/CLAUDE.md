# 002_multiple_operations

Tests that a single patch can perform multiple operations: add a file, delete a file, and update a file simultaneously.

## Files

- `patch.txt` -- Patch that adds `nested/new.txt`, deletes `delete.txt`, and updates `modify.txt` (replacing "line2" with "changed").
- `input/` -- Initial state with `delete.txt` and `modify.txt`.
- `expected/` -- Expected state with `nested/new.txt` and modified `modify.txt`; `delete.txt` is gone.
