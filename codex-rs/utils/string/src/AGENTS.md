# codex-rs/utils/string/src/

This file applies to `codex-rs/utils/string/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-utils-string` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-utils-string`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source directory for the `codex-utils-string` crate.

### Key files

- `lib.rs` -- single-file implementation containing:
  - `take_bytes_at_char_boundary(s: &str, maxb: usize) -> &str` -- scans char indices to find the longest prefix within the byte budget
  - `take_last_bytes_at_char_boundary(s: &str, maxb: usize) -> &str` -- reverse scan for suffix within byte budget
  - `sanitize_metric_tag_value(value: &str) -> String` -- replaces invalid chars (non-ASCII-alphanumeric except `.`, `_`, `-`, `/`) with `_`, trims underscores, caps at 256 chars, returns `"unspecified"` for empty/non-alphanumeric results
  - `find_uuids(s: &str) -> Vec<String>` -- uses a `OnceLock<regex_lite::Regex>` to find all standard UUID patterns
  - `normalize_markdown_hash_location_suffix(suffix: &str) -> Option<String>` -- parses `#L<line>[C<col>][-L<line>[C<col>]]` and converts to `:<line>[:<col>][-<line>[:<col>]]`
  - `parse_markdown_hash_location_point` -- internal helper for parsing L/C coordinates
  - Tests for UUID finding, metric sanitization, and markdown location normalization
