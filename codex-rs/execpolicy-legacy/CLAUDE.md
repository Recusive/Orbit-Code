# codex-rs/execpolicy-legacy/

Legacy exec policy engine for validating proposed shell commands. This is the older argument-aware policy system that understands program-specific argument semantics (flags, options, positional args, sed commands, etc.).

## What this folder does

Provides a Starlark-based policy engine that goes beyond simple prefix matching to parse and validate individual program arguments. Unlike the newer `codex-execpolicy` (prefix-only), this engine understands argument types, option semantics, and can determine whether a command might write files. It includes a built-in default policy (`default.policy`) and a CLI binary for checking commands.

## What it plugs into

- **codex-core** -- uses this alongside the newer execpolicy for backward-compatible command validation
- Standalone CLI (`codex-execpolicy-legacy check`) for manual evaluation

## Imports from

- `starlark`: Starlark language interpreter for policy DSL parsing
- `allocative`: memory-aware types for Starlark values
- `multimap`: rule indexing by program name
- `regex-lite`: lightweight regex for argument matching
- `path-absolutize`: path normalization
- `serde`, `serde_json`, `serde_with`: serialization

## Exports to

- `Policy` -- compiled policy with `check(&ExecCall) -> Result<MatchedExec>`
- `PolicyParser` -- Starlark-based parser for `.policy` files
- `ExecCall` -- input: program name + args
- `MatchedExec` -- result: `Match { exec: ValidExec }` or `Forbidden { reason, cause }`
- `ValidExec` -- validated execution with matched args, flags, opts, and `might_write_files()` check
- `ExecvChecker` -- lower-level checker interface
- `ProgramSpec`, `Opt`, `ArgType`, `ArgMatcher`, `PositionalArg` -- DSL types for defining program argument specs
- `get_default_policy()` -- loads the built-in default policy
- `parse_sed_command` -- sed command parser for sed-specific policy rules

## Key files

- `Cargo.toml` -- crate metadata; binary `codex-execpolicy-legacy`, library `codex_execpolicy_legacy`
- `README.md` -- documentation
- `build.rs` -- build script (if present)
- `src/main.rs` -- CLI entry point with `check` and `check-json` subcommands; outputs JSON with safe/match/forbidden/unverified status
- `src/lib.rs` -- module declarations and public re-exports; includes `DEFAULT_POLICY` (embedded `default.policy`)
- `src/policy.rs` -- `Policy` struct with `check()` method
- `src/policy_parser.rs` -- Starlark parser for the legacy DSL
- `src/program.rs` -- `ProgramSpec`, `MatchedExec`, `Forbidden` types
- `src/valid_exec.rs` -- `ValidExec` with `MatchedArg`, `MatchedFlag`, `MatchedOpt`; `might_write_files()`
- `src/execv_checker.rs` -- `ExecvChecker` implementation
- `src/exec_call.rs` -- `ExecCall` input type
- `src/opt.rs` -- `Opt` type for option definitions
- `src/arg_type.rs` -- `ArgType` enum for argument classification
- `src/arg_matcher.rs` -- `ArgMatcher` for pattern-based argument validation
- `src/arg_resolver.rs` -- argument resolution logic
- `src/sed_command.rs` -- sed command parsing
- `src/error.rs` -- error types
- `src/default.policy` -- built-in default policy (embedded via `include_str!`)
