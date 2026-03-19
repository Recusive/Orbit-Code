# codex-rs/cloud-tasks/

Interactive TUI for browsing, applying, and managing Codex Cloud tasks.

## What this folder does

Provides the `codex cloud` subcommand -- a Ratatui-based terminal UI for listing cloud tasks, viewing diffs, applying changes locally, submitting new tasks, and showing task status. Supports environment filtering, multi-attempt (best-of-N) task creation, and scrollable diff viewing.

## Where it plugs in

- Invoked from `codex-cli` as the `cloud` / `cloud-tasks` subcommand
- Uses `codex-cloud-tasks-client` for backend API communication (with both `online` and `mock` features)
- Uses `codex-tui` for shared TUI components
- Uses `codex-login` for authentication
- Uses `codex-core` for configuration

## Imports from

- `codex-cloud-tasks-client` -- `CloudBackend`, `HttpClient`, `MockClient`, task types
- `codex-tui` -- shared TUI components
- `codex-login` -- login flows
- `codex-core` -- configuration, auth
- `codex-client` -- reqwest client builder
- `codex-utils-cli` -- CLI config override parsing
- `ratatui` / `crossterm` -- terminal UI
- `clap` -- CLI argument parsing
- `owo-colors` / `supports-color` -- colored output

## Exports to

Public API:

- `Cli` -- clap command struct used by `codex-cli`
- `run_main(cli, codex_linux_sandbox_exe)` -- entry point
- `env_detect` / `scrollable_diff` / `util` modules

## Key files

| File | Role |
|------|------|
| `Cargo.toml` | Crate manifest; depends on `codex-cloud-tasks-client`, `codex-tui`, `ratatui`, `crossterm` |
| `src/lib.rs` | Entry point, task listing/application logic, `run_main` |
| `src/cli.rs` | `Cli` struct with subcommands: `Exec`, `Status`, `List`, `Apply`, `Diff` |
| `src/app.rs` | Ratatui app state and event loop |
| `src/ui.rs` | UI rendering logic |
| `src/new_task.rs` | New task creation flow |
| `src/env_detect.rs` | Environment detection helpers |
| `src/scrollable_diff.rs` | Scrollable diff viewer widget |
| `src/util.rs` | Utility functions (time formatting, error logging, user agent) |
| `tests/` | Test directory |
