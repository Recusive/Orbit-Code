# 001_add_file

Tests that a patch with `*** Add File` creates a new file with the specified contents.

## Files

- `patch.txt` -- Patch that adds `bar.md` with content "This is a new file".
- `expected/bar.md` -- Expected output file after the patch is applied.

This scenario has no `input/` directory since no pre-existing files are needed.
