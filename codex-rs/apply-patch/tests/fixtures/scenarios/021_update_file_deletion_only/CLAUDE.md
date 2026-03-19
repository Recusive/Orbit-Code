# 021_update_file_deletion_only

Tests that an Update File hunk can remove lines without adding any new lines (deletion-only change).

## Files

- `patch.txt` -- Patch that updates `lines.txt`, removing "line2" while keeping "line1" and "line3" as context.
- `input/lines.txt` -- Original file with three lines.
- `expected/lines.txt` -- File with "line2" removed.
