# codex-rs/utils/fuzzy-match/

This file applies to `codex-rs/utils/fuzzy-match/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-utils-fuzzy-match` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-utils-fuzzy-match`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Crate `codex-utils-fuzzy-match` -- case-insensitive fuzzy subsequence matching.

### What this folder does

Implements a simple fuzzy matcher that finds a case-insensitive subsequence of needle characters in a haystack string. Returns matched character indices (in the original haystack) and a score (lower is better) that rewards contiguous matches and prefix matches.

### Key types and functions

- `fuzzy_match(haystack, needle) -> Option<(Vec<usize>, i32)>` -- returns matched indices and score; `None` if needle is not a subsequence
- `fuzzy_indices(haystack, needle) -> Option<Vec<usize>>` -- convenience wrapper returning just indices
- Scoring: contiguous matches score 0, spread matches penalized by window gap, prefix matches get a -100 bonus

### Imports from

No external dependencies (std only).

### Exports to

Used by `codex-tui` for fuzzy filtering in interactive selection UIs.

### Key files

- `Cargo.toml` -- crate metadata (no dependencies)
- `src/lib.rs` -- `fuzzy_match`, `fuzzy_indices`, Unicode-aware lowercase mapping, and comprehensive tests including Unicode edge cases
