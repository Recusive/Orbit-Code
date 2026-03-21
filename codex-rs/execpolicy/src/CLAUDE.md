# codex-rs/execpolicy/src/

Source for the `orbit-code-execpolicy` crate -- Starlark-based prefix policy engine with CLI binary.

## Module Layout
- **Policy engine** (`policy.rs`): `Policy` struct with `check()`, `merge_overlay()`, `matches_for_command()`; stores rules in `MultiMap<String, RuleRef>` by program name
- **Rule system** (`rule.rs`, `decision.rs`): `Rule` trait, `PrefixRule`, `NetworkRule`, `Decision` enum with ordering, match/not-match example validation
- **Parser** (`parser.rs`): `PolicyParser` wrapping the Starlark interpreter with custom built-in functions (`prefix_rule`, `network_rule`, `host_executable`)
- **CLI** (`main.rs`, `execpolicycheck.rs`): `ExecPolicyCheckCommand` for `check` subcommand -- loads policies and outputs JSON decisions
- **File amendment** (`amend.rs`): Helpers for appending rules to existing `.codexpolicy` files on disk
- **Utilities** (`error.rs`, `executable_name.rs`): Error types with source location tracking; basename extraction for executable path resolution
