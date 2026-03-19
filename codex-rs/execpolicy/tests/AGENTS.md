# codex-rs/execpolicy/tests/

This file applies to `codex-rs/execpolicy/tests/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-execpolicy` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-execpolicy`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Tests for the `codex-execpolicy` crate.

### What this folder does

Contains comprehensive integration tests for the exec policy engine, covering policy parsing, rule matching, network rules, host executable resolution, overlay merging, and the CLI check command.

### Key files and their roles

- `basic.rs` -- Extensive test suite covering: prefix rule matching (exact, prefix, multi-token), alt patterns, decision ordering, heuristics fallback, network rules (allow/deny/protocol variants, host normalization, IPv6), host executable path resolution, policy overlay merging, `compiled_network_domains()`, `get_allowed_prefixes()`, `add_prefix_rule()`/`add_network_rule()`, Starlark parser edge cases, match/not-match example validation, `ExecPolicyCheckCommand` CLI output, and the `blocking_append_*` amend functions.

### Imports from

- `codex_execpolicy`: Policy, PolicyParser, Decision, Evaluation, MatchOptions, RuleMatch, RuleRef, NetworkRuleProtocol, ExecPolicyCheckCommand, blocking_append_allow_prefix_rule, rule submodule types
- `codex_utils_absolute_path`: AbsolutePathBuf
- `tempfile`, `pretty_assertions`: test utilities
