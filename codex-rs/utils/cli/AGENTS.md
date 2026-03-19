# codex-rs/utils/cli/

This file applies to `codex-rs/utils/cli/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-utils-cli` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-utils-cli`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Crate `codex-utils-cli` -- shared CLI argument types and configuration override support.

### What this folder does

Provides reusable clap-derived argument types for the `--approval-mode`, `--sandbox` (`-s`), and `-c key=value` configuration override flags. These types are shared between the TUI, exec, and other CLI entry points to ensure consistent argument parsing.

### Key types and functions

- `ApprovalModeCliArg` -- clap `ValueEnum` mapping CLI flags to `AskForApproval` protocol variants
- `SandboxModeCliArg` -- clap `ValueEnum` mapping CLI flags to `SandboxMode` config variants
- `CliConfigOverrides` -- clap `Parser` struct capturing `-c key=value` overrides; includes `parse_overrides()` and `apply_on_value()` for merging onto TOML config trees
- `format_env_display` -- formats environment variable maps for display with masked values

### Imports from

- `clap` -- CLI argument parsing
- `codex-protocol` -- `AskForApproval`, `SandboxPolicy`, `SandboxMode` types
- `serde`, `toml` -- configuration value parsing

### Exports to

Consumed by `codex-cli` (TUI), `codex-exec` (headless), and `codex-app-server` for CLI argument handling.

### Key files

- `Cargo.toml` -- crate metadata and dependencies
- `src/lib.rs` -- module declarations and re-exports
- `src/approval_mode_cli_arg.rs` -- `ApprovalModeCliArg` enum
- `src/sandbox_mode_cli_arg.rs` -- `SandboxModeCliArg` enum
- `src/config_override.rs` -- `CliConfigOverrides` struct with TOML parsing and override application
- `src/format_env_display.rs` -- environment variable display formatting
