# codex-rs/execpolicy/

This file applies to `codex-rs/execpolicy/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-execpolicy` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-execpolicy`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Prefix-based Starlark policy engine for evaluating whether shell commands should be allowed, prompted, or forbidden. This is the current (non-legacy) exec policy system.

### What this folder does

Provides the `codex-execpolicy` library and CLI binary. Policies are written in a Starlark DSL (`.codexpolicy` files) using built-in functions like `prefix_rule()`, `network_rule()`, and `host_executable()`. The engine evaluates commands against these rules and returns a decision (Allow, Prompt, or Forbidden) with matched rule details.

### What it plugs into

- **codex-core** -- uses this crate to evaluate commands proposed by the agent before execution
- **codex-config** -- policy files are discovered from the config layer stack (global, project, local)
- Standalone CLI (`codex-execpolicy check`) for manual policy evaluation

### Imports from

- `codex-utils-absolute-path`: `AbsolutePathBuf` for host executable path resolution
- `starlark`: Starlark language interpreter for parsing `.codexpolicy` files
- `multimap`: `MultiMap` for indexing rules by program name
- `shlex`: command tokenization
- `serde`, `serde_json`: serialization of decisions and rule matches

### Exports to

- `Policy` -- the compiled policy object with `check()`, `check_multiple()`, `matches_for_command()`, `add_prefix_rule()`, `add_network_rule()`, `merge_overlay()`
- `PolicyParser` -- Starlark-based parser that builds a `Policy` from `.codexpolicy` file contents
- `Decision` -- enum: Allow, Prompt, Forbidden
- `Rule`, `RuleRef`, `RuleMatch`, `PrefixRule`, `NetworkRuleProtocol` -- rule types and matching results
- `Evaluation` -- combined decision + matched rules
- `MatchOptions` -- options for command matching (e.g., resolve_host_executables)
- `ExecPolicyCheckCommand` -- CLI subcommand for evaluating commands
- `blocking_append_allow_prefix_rule`, `blocking_append_network_rule` -- helpers for amending policy files
- Error types: `Error`, `ErrorLocation`, `TextPosition`, `TextRange`

### Key files

- `Cargo.toml` -- crate metadata; binary `codex-execpolicy`, library `codex_execpolicy`
- `README.md` -- policy language documentation
- `src/main.rs` -- CLI entry point with `check` subcommand
- `src/lib.rs` -- public re-exports
- `src/policy.rs` -- `Policy` struct and `Evaluation`: rule storage, command matching, network domain compilation, overlay merging
- `src/decision.rs` -- `Decision` enum (Allow < Prompt < Forbidden ordering)
- `src/rule.rs` -- `Rule` trait, `PrefixRule`, `PrefixPattern`, `PatternToken`, `NetworkRule`, `RuleMatch`, `RuleRef`; match/not-match example validation
- `src/parser.rs` -- `PolicyParser`: Starlark evaluator with custom built-in functions (prefix_rule, network_rule, host_executable)
- `src/execpolicycheck.rs` -- `ExecPolicyCheckCommand` CLI and `load_policies()` helper
- `src/amend.rs` -- file-level helpers for appending rules to existing policy files
- `src/error.rs` -- error types with source location tracking
- `src/executable_name.rs` -- basename extraction for executable path resolution
