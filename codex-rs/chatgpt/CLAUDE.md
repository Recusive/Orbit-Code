# codex-rs/chatgpt/

ChatGPT backend integration for Codex -- task retrieval, diff application, and connector management.

## What this folder does

This crate provides functionality for interacting with the ChatGPT backend API (via `/wham/` endpoints). It handles authentication token management, fetching cloud tasks, applying diffs from task results to local repositories, and listing/managing connectors (third-party app integrations).

## Where it plugs in

- Used by the CLI (`codex-cli`) for the `apply` subcommand and connector listing
- Depends on `codex-core` for config, auth, connectors, and HTTP client creation
- Depends on `codex-connectors` for connector caching and directory listing
- Depends on `codex-git` for applying git patches
- Depends on `codex-utils-cli` for CLI config override parsing

## Imports from

- `codex-core` -- `Config`, `AuthManager`, `TokenData`, `AuthCredentialsStoreMode`, `create_client`, connectors helpers
- `codex-connectors` -- `AllConnectorsCacheKey`, `DirectoryListResponse`, caching and listing functions
- `codex-git` -- `ApplyGitRequest`, `apply_git_patch`
- `codex-utils-cli` -- `CliConfigOverrides`
- `codex-utils-cargo-bin` -- `find_resource!` macro (tests)
- `clap` -- CLI argument parsing for `ApplyCommand`
- `serde` / `serde_json` -- JSON deserialization
- `tokio` -- async runtime

## Exports to

Public API from `lib.rs`:

- `apply_command` module -- `ApplyCommand` (clap struct), `run_apply_command`, `apply_diff_from_task`
- `connectors` module -- `list_connectors`, `list_all_connectors`, `list_cached_all_connectors`, `merge_connectors_with_accessible`, `connectors_for_plugin_apps`, and re-exports from `codex-core::connectors`
- `get_task` module -- `GetTaskResponse`, `AssistantTurn`, `OutputItem`, `PrOutputItem`, `OutputDiff`, `get_task`

## Key files

| File | Role |
|------|------|
| `Cargo.toml` | Crate manifest; depends on `codex-core`, `codex-connectors`, `codex-git`, `codex-utils-cli` |
| `src/lib.rs` | Module declarations |
| `src/chatgpt_client.rs` | `chatgpt_get_request` -- authenticated GET to the ChatGPT backend API |
| `src/chatgpt_token.rs` | Global token management via `LazyLock<RwLock<Option<TokenData>>>` |
| `src/connectors.rs` | Connector listing, caching, merging, filtering, and plugin app integration |
| `src/get_task.rs` | Task fetching and response types (`GetTaskResponse`, `AssistantTurn`, `OutputItem`) |
| `src/apply_command.rs` | `ApplyCommand` clap struct and `apply_diff_from_task` which extracts a diff and applies it via `codex-git` |
| `tests/` | Integration tests |
