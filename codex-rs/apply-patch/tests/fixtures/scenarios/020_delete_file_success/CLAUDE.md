# 020_delete_file_success

Tests that `*** Delete File` successfully removes an existing file while leaving other files untouched.

## Files

- `patch.txt` -- Patch that deletes `obsolete.txt`.
- `input/obsolete.txt` -- File to be deleted.
- `input/keep.txt` -- File that should remain.
- `expected/keep.txt` -- Same as input; `obsolete.txt` is absent.
