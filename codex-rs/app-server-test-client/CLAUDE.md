# app-server-test-client

## Purpose

A CLI test client for the `orbit-code-app-server`. Provides a command-line interface for manually or programmatically exercising the app-server's JSON-RPC API over stdio or WebSocket transports. Used for development testing, debugging, and as a harness for integration test scripts.

## What It Plugs Into

- Connects to a running `orbit-code-app-server` instance via stdio (launching as a subprocess) or WebSocket (`--url ws://...`).
- Performs the initialize/initialized handshake, then dispatches subcommands that map to app-server JSON-RPC methods.

## Key Features

- **Transport modes:** Stdio (default, launches app-server as child process) or WebSocket (`--url`).
- **Subcommands:** Thread lifecycle (start, resume, read, list, fork, rollback, archive, unarchive), turn management (start, interrupt, steer), model listing, account/login, config read/write, plugin management, command execution, MCP elicitation, review, and more.
- **Auto-approval:** Optional `--auto-approve` flag that automatically approves command execution and file change requests.
- **Interactive mode:** Reads user input from stdin for turn prompts and displays streamed agent output.

## Key Files

| File | Role |
|------|------|
| `Cargo.toml` | Crate manifest |
| `src/main.rs` | Binary entry point. Builds a single-threaded Tokio runtime and delegates to `lib::run()`. |
| `src/lib.rs` | Core client logic. Defines CLI args via `clap` with subcommands, implements the JSON-RPC message loop, request/response routing, notification handling, and auto-approval logic. |
| `scripts/live_elicitation_hold.sh` | Shell script for testing MCP elicitation hold behavior. Increments an elicitation counter on a thread, sleeps, then decrements it. |

## Imports From

- `orbit-code-app-server-protocol` -- All JSON-RPC and typed protocol types for constructing requests and parsing responses.
- `orbit-code-core` -- Config loading.
- `orbit-code-protocol` -- Shared types.
- `orbit-code-otel` -- Observability setup.
- `orbit-code-utils-cli` -- CLI utilities.
- `tungstenite` -- Synchronous WebSocket client.

## Exports To

- Used as a standalone binary. The `lib.rs` exports `run()` for the binary entry point.
- Referenced by test scripts and CI workflows for end-to-end testing.
