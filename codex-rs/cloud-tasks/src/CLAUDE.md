# codex-rs/cloud-tasks/src/

Source directory for the `codex-cloud-tasks` crate.

## What this folder does

Contains the implementation of the Codex Cloud tasks TUI and CLI subcommands for listing, viewing, applying, and creating cloud tasks.

## Key files

| File | Role |
|------|------|
| `lib.rs` | Module declarations, `Cli` re-export, `run_main` entry point, task listing/apply orchestration |
| `cli.rs` | `Cli` clap struct with `Command` enum: `Exec`, `Status`, `List`, `Apply`, `Diff` subcommands |
| `app.rs` | Ratatui app state management and terminal event loop |
| `ui.rs` | Terminal UI rendering (task list, diff view, status display) |
| `new_task.rs` | New task creation: prompt input, environment selection, branch detection, multi-attempt support |
| `env_detect.rs` | Git environment detection helpers |
| `scrollable_diff.rs` | Scrollable unified diff viewer widget |
| `util.rs` | Shared utilities: relative time formatting, error log appending, user agent suffix |
