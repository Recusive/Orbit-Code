# codex-rs/.config/

Configuration directory for the cargo-nextest test runner.

## What this folder does

Contains configuration for `cargo-nextest`, the test runner used by the workspace instead of the default `cargo test`.

## Key files

- `nextest.toml` -- Test runner configuration:
  - **Default slow-timeout**: 15 seconds per test period, terminate after 2 periods (30s total). Tests should be fixed rather than increasing this.
  - **Overrides**: Specific tests get longer timeouts:
    - `rmcp_client` and `humanlike_typing_1000_chars_appears_live_no_placeholder`: 1 minute period, terminate after 4 periods
    - `approval_matrix_covers_all_modes`: 30 second period, terminate after 2 periods
  - **Test groups** (serial execution):
    - `app_server_protocol_codegen`: TypeScript/JSON schema codegen tests run single-threaded to avoid file conflicts
    - `app_server_integration`: Integration tests that spawn app-server subprocesses run single-threaded

## What it plugs into

- `cargo nextest run` reads this file automatically from the `.config/` directory
- CI pipelines use nextest for test execution with these timeout and concurrency constraints

## Imports from / exports to

- No code imports; this is a tool configuration file
- Consumed by `cargo-nextest` at test runtime
