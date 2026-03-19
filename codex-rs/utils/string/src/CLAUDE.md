# codex-rs/utils/string/src/

Source directory for the `codex-utils-string` crate.

## Key files

- `lib.rs` -- single-file implementation containing:
  - `take_bytes_at_char_boundary(s: &str, maxb: usize) -> &str` -- scans char indices to find the longest prefix within the byte budget
  - `take_last_bytes_at_char_boundary(s: &str, maxb: usize) -> &str` -- reverse scan for suffix within byte budget
  - `sanitize_metric_tag_value(value: &str) -> String` -- replaces invalid chars (non-ASCII-alphanumeric except `.`, `_`, `-`, `/`) with `_`, trims underscores, caps at 256 chars, returns `"unspecified"` for empty/non-alphanumeric results
  - `find_uuids(s: &str) -> Vec<String>` -- uses a `OnceLock<regex_lite::Regex>` to find all standard UUID patterns
  - `normalize_markdown_hash_location_suffix(suffix: &str) -> Option<String>` -- parses `#L<line>[C<col>][-L<line>[C<col>]]` and converts to `:<line>[:<col>][-<line>[:<col>]]`
  - `parse_markdown_hash_location_point` -- internal helper for parsing L/C coordinates
  - Tests for UUID finding, metric sanitization, and markdown location normalization
