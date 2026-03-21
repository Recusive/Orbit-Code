# codex-rs/debug-client/src/

Source directory for the `orbit-code-debug-client` binary.

## What this folder does

Contains the implementation of the minimal interactive debug client for testing the Codex app server protocol v2.

## Key files

| File | Role |
|------|------|
| `main.rs` | `Cli` clap struct with flags for `--codex-bin`, `--thread-id`, `--approval-policy`, `--auto-approve`, `--final-only`, `--model`, `--cwd`; main loop: spawns app server, performs initialize handshake, reads stdin for user input, dispatches commands |
| `client.rs` | `AppServerClient` -- spawns `codex app-server` subprocess; `build_thread_start_params` / `build_thread_resume_params` -- constructs JSON-RPC params; sends turn messages, handles approval responses |
| `commands.rs` | `parse_input` -- parses user text into `InputAction` (send turn, execute command); `UserCommand` enum: `Help`, `New`, `Resume`, `Use`, `RefreshThread`, `Quit` |
| `output.rs` | `Output` -- handles printing server JSON-RPC messages to stdout and client messages to stderr; `--final-only` filtering |
| `reader.rs` | Background thread that reads lines from the app server's stdout and sends parsed JSON values as `ReaderEvent`s |
| `state.rs` | `ReaderEvent` enum -- `Line(Value)` or `Eof`; shared state types for the reader thread |
