# 020_whitespace_padded_patch_marker_lines

Tests that `*** Begin Patch` and `*** End Patch` lines with trailing whitespace on the marker lines themselves are tolerated.

## Files

- `patch.txt` -- Patch with trailing whitespace on the Begin Patch line and leading space on the End Patch line.
- `input/file.txt` -- Original file with "one".
- `expected/file.txt` -- Updated file with "two".
