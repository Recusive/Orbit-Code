# codex-rs/execpolicy-legacy/src/

Source for the `orbit-code-execpolicy-legacy` crate -- argument-aware Starlark policy engine with embedded default policy.

## Module Layout
- **Policy engine** (`policy.rs`, `policy_parser.rs`): `Policy` struct with `check()` method; Starlark parser for the legacy DSL with program-specific built-in functions
- **Program specs** (`program.rs`, `opt.rs`, `arg_type.rs`): `ProgramSpec` defining accepted arguments; `Opt` for option definitions; `ArgType` enum classifying arguments for write-safety analysis
- **Matching** (`execv_checker.rs`, `arg_matcher.rs`, `arg_resolver.rs`): `ExecvChecker` matches calls against specs; `ArgMatcher` validates patterns via regex; `PositionalArg` resolution logic
- **Validation** (`valid_exec.rs`): `ValidExec` with `might_write_files()` check; `MatchedArg`, `MatchedFlag`, `MatchedOpt` tracking
- **Specialized** (`sed_command.rs`): sed command parser for sed-specific policy rules
- **CLI** (`main.rs`): Binary with `check` and `check-json` subcommands; `default.policy` embedded via `include_str!`
