# codex-rs/utils/string/

This file applies to `codex-rs/utils/string/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-utils-string` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-utils-string`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Crate `codex-utils-string` -- string utility functions for byte-boundary truncation, UUID extraction, and metric sanitization.

### What this folder does

Provides small, focused string manipulation utilities used across the Codex codebase for output truncation, metric tag sanitization, UUID extraction, and markdown location suffix normalization.

### Key types and functions

- `take_bytes_at_char_boundary(s, maxb)` -- truncate a string prefix to a byte budget, respecting char boundaries
- `take_last_bytes_at_char_boundary(s, maxb)` -- take a string suffix within a byte budget, respecting char boundaries
- `sanitize_metric_tag_value(value)` -- replace non-alphanumeric/non-separator characters with `_`; trim; cap at 256 chars; return `"unspecified"` for invalid values
- `find_uuids(s)` -- extract all UUID strings matching the standard 8-4-4-4-12 hex pattern
- `normalize_markdown_hash_location_suffix(suffix)` -- convert markdown `#L74C3-L76C9` style suffixes to terminal-friendly `:74:3-76:9` format

### Imports from

- `regex-lite` -- lightweight regex for UUID extraction

### Exports to

Used by `codex-core` for output truncation, `codex-tui` for display formatting, and telemetry code for metric tag sanitization.

### Key files

- `Cargo.toml` -- crate metadata; depends on `regex-lite`
- `src/lib.rs` -- all functions, `OnceLock`-cached UUID regex, and tests
