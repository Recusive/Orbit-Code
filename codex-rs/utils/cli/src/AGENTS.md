# codex-rs/utils/cli/src/

This file applies to `codex-rs/utils/cli/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-utils-cli` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-utils-cli`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source directory for the `codex-utils-cli` crate.

### Key files

- `lib.rs` -- module declarations; re-exports `ApprovalModeCliArg`, `CliConfigOverrides`, `SandboxModeCliArg`
- `approval_mode_cli_arg.rs` -- `ApprovalModeCliArg` enum (Untrusted, OnFailure, OnRequest, Never) with `From` impl converting to `AskForApproval`
- `sandbox_mode_cli_arg.rs` -- `SandboxModeCliArg` enum (ReadOnly, WorkspaceWrite, DangerFullAccess) with `From` impl converting to `SandboxMode`
- `config_override.rs` -- `CliConfigOverrides` struct:
  - Captures `-c key=value` flags via `clap::ArgAction::Append`
  - `parse_overrides()` splits on first `=`, parses RHS as TOML (falling back to raw string)
  - `apply_on_value()` applies dotted-path overrides onto a `toml::Value` tree
  - Canonicalizes legacy key aliases (e.g., `use_legacy_landlock` -> `features.use_legacy_landlock`)
- `format_env_display.rs` -- `format_env_display()` function that formats env vars with masked values for display
