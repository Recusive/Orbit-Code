# app-server-test-client/src

## Purpose

Source code for the `orbit-code-app-server-test-client` binary. Implements a CLI tool for exercising the app-server JSON-RPC API.

## Key Files

| File | Role |
|------|------|
| `main.rs` | Binary entry point. Creates a single-threaded Tokio runtime and calls `orbit_code_app_server_test_client::run()`. Minimal -- all logic is in `lib.rs`. |
| `lib.rs` | Core client implementation. Defines the CLI structure using `clap` with subcommands for all major app-server operations. Key components: |

### `lib.rs` Details

- **CLI structure:** `Args` struct with `--url` (WebSocket endpoint), `--auto-approve`, `--log-level`, and a `Subcommand` enum covering: `ThreadStart`, `ThreadResume`, `ThreadRead`, `ThreadList`, `ThreadFork`, `ThreadRollback`, `ThreadArchive`, `ThreadUnarchive`, `TurnStart`, `TurnInterrupt`, `TurnSteer`, `ModelList`, `Login`, `GetAccount`, `GetAccountRateLimits`, `ConfigRead`, `ConfigValueWrite`, `Review`, `CommandExec`, `PluginInstall`, `PluginList`, `PluginRead`, `PluginUninstall`, `ThreadIncrementElicitation`, `ThreadDecrementElicitation`, and more.
- **Transport:** Connects via synchronous WebSocket (`tungstenite`) or spawns app-server as a child process with stdio transport.
- **Message loop:** `run_message_loop()` reads JSON-RPC messages, routes responses to pending requests, handles server requests (approval prompts) with optional auto-approval, and displays server notifications.
- **Auto-approval:** When `--auto-approve` is set, automatically accepts command execution and file change approval requests.
- **Interactive input:** For `TurnStart`, reads user prompts from stdin and streams agent responses to stdout.

## What It Plugs Into

- Connects to `orbit-code-app-server` via WebSocket or stdio.
- Uses `orbit-code-app-server-protocol` for all message types.

## Imports From

- `orbit-code-app-server-protocol` -- All JSON-RPC types, `ClientRequest`, `ServerRequest`, `ServerNotification`, individual request/response param structs.
- `orbit-code-core` -- Config loading for stdio mode.
- `orbit-code-protocol` -- Shared protocol types.
- `tungstenite` -- Synchronous WebSocket client.
- `clap` -- CLI argument parsing.
- `serde_json` -- JSON serialization.

## Exports To

- `lib.rs` exports `pub async fn run()` consumed by `main.rs`.
