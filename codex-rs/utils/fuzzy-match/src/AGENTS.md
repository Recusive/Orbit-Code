# codex-rs/utils/fuzzy-match/src/

This file applies to `codex-rs/utils/fuzzy-match/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-utils-fuzzy-match` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-utils-fuzzy-match`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source directory for the `codex-utils-fuzzy-match` crate.

### Key files

- `lib.rs` -- single-file implementation containing:
  - `fuzzy_match(haystack: &str, needle: &str) -> Option<(Vec<usize>, i32)>` -- the core algorithm:
    - Lowercases both strings, maintaining a mapping from lowered char indices back to original haystack char indices
    - Greedily matches needle characters as a subsequence in the lowered haystack
    - Scores based on span window minus needle length; -100 bonus for prefix matches
    - Empty needle matches everything with `i32::MAX` score
  - `fuzzy_indices(haystack, needle) -> Option<Vec<usize>>` -- returns only the deduped, sorted indices
  - Tests covering ASCII, Unicode (Turkish dotted-I, German sharp-s), contiguous vs spread matches, prefix bonuses, and multi-char lowercase expansion
