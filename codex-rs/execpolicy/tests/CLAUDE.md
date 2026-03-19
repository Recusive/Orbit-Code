# codex-rs/execpolicy/tests/

Tests for the `codex-execpolicy` crate.

## What this folder does

Contains comprehensive integration tests for the exec policy engine, covering policy parsing, rule matching, network rules, host executable resolution, overlay merging, and the CLI check command.

## Key files and their roles

- `basic.rs` -- Extensive test suite covering: prefix rule matching (exact, prefix, multi-token), alt patterns, decision ordering, heuristics fallback, network rules (allow/deny/protocol variants, host normalization, IPv6), host executable path resolution, policy overlay merging, `compiled_network_domains()`, `get_allowed_prefixes()`, `add_prefix_rule()`/`add_network_rule()`, Starlark parser edge cases, match/not-match example validation, `ExecPolicyCheckCommand` CLI output, and the `blocking_append_*` amend functions.

## Imports from

- `codex_execpolicy`: Policy, PolicyParser, Decision, Evaluation, MatchOptions, RuleMatch, RuleRef, NetworkRuleProtocol, ExecPolicyCheckCommand, blocking_append_allow_prefix_rule, rule submodule types
- `codex_utils_absolute_path`: AbsolutePathBuf
- `tempfile`, `pretty_assertions`: test utilities
