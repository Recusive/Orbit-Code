# codex-rs/exec/tests/suite/

Integration test modules for the `codex-exec` binary.

## What this folder does

Contains individual integration test modules that exercise end-to-end behavior of the `codex-exec` binary. Each module focuses on a specific feature or CLI flag. All modules are aggregated via `mod.rs` into the single test binary defined by `tests/all.rs`.

## Key files and their roles

- `mod.rs` -- Module aggregator; imports all test modules in this directory.
- `add_dir.rs` -- Tests `--add-dir` flag for additional writable directories.
- `apply_patch.rs` -- Tests apply_patch tool execution and output.
- `auth_env.rs` -- Tests authentication environment variable handling.
- `ephemeral.rs` -- Tests `--ephemeral` flag for non-persistent sessions.
- `mcp_required_exit.rs` -- Tests exit behavior when a required MCP server fails to initialize.
- `originator.rs` -- Tests that the originator is correctly set to "codex_exec".
- `output_schema.rs` -- Tests `--output-schema` flag for structured JSON output.
- `resume.rs` -- Tests `resume` subcommand for session resumption.
- `sandbox.rs` -- Tests sandbox mode configuration and behavior.
- `server_error_exit.rs` -- Tests that server errors cause non-zero exit codes.

## Imports from

- `codex_exec` (library crate)
- `core_test_support` (test infrastructure)
- `assert_cmd`, `predicates` (CLI testing utilities)
- `codex_apply_patch`, `codex_utils_cargo_bin` (test helpers)

## What it plugs into

- Aggregated by `tests/all.rs` via `mod suite;`
