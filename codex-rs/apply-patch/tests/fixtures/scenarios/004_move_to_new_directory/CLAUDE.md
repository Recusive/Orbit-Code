# 004_move_to_new_directory

Tests that `*** Move to:` renames/moves a file to a new directory path, creating intermediate directories as needed, and removes the original file.

## Files

- `patch.txt` -- Patch that updates and moves `old/name.txt` to `renamed/dir/name.txt`, replacing "old content" with "new content".
- `input/old/` -- Contains `name.txt` (the file to move) and `other.txt` (untouched).
- `expected/` -- Contains `old/other.txt` (unchanged) and `renamed/dir/name.txt` (moved and updated); `old/name.txt` is absent.
