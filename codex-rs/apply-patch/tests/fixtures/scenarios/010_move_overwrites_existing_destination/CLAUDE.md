# 010_move_overwrites_existing_destination

Tests that a Move To operation overwrites a file that already exists at the destination path.

## Files

- `patch.txt` -- Patch that updates and moves `old/name.txt` to `renamed/dir/name.txt`, replacing "from" with "new". The destination already has an existing file.
- `input/` -- Contains `old/name.txt`, `old/other.txt`, and a pre-existing `renamed/dir/name.txt`.
- `expected/` -- Contains `old/other.txt` (unchanged) and `renamed/dir/name.txt` (overwritten with new content).
