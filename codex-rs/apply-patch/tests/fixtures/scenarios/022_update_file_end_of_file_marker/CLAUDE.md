# 022_update_file_end_of_file_marker

Tests that the `*** End of File` marker anchors a change to the end of the file, ensuring the replacement happens at the file's tail.

## Files

- `patch.txt` -- Patch that updates `tail.txt` with an `*** End of File` marker, replacing "second" with "second updated" at the end of the file.
- `input/tail.txt` -- Original file with "first" and "second".
- `expected/tail.txt` -- Updated file with "second updated" at the end.
