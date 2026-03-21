# codex-rs/exec/

Headless (non-interactive) CLI for running the Orbit Code agent. Designed for automation, CI pipelines, and scripted usage where no TUI is needed.

## Build & Test

```bash
cargo build -p orbit-code-exec         # Build the binary
cargo test -p orbit-code-exec          # Run tests (unit + integration in tests/)
just fmt                               # Format after changes
just fix -p orbit-code-exec            # Clippy
```

## Architecture

The exec crate wraps the agent engine for non-interactive use:

1. **Entry**: `main.rs` uses `orbit-code-arg0` for multi-binary dispatch (when invoked as `orbit-code-linux-sandbox`, runs sandbox logic instead). Otherwise parses CLI and calls `run_main()`.

2. **Session lifecycle** (`lib.rs`):
   - `run_main()` loads config, initializes OTEL, creates an `InProcessAppServerClient`, resolves the prompt (from CLI arg or stdin with BOM handling), and enters the event loop
   - `run_exec_session()` creates an `EventProcessor`, starts a thread via `ThreadStartParams`, sends a turn, and processes events until completion or shutdown
   - Handles server requests (approval prompts, auth token refresh, MCP elicitation) inline

3. **Two output modes** selected by `--json` flag:
   - **Human** (`event_processor_with_human_output.rs`): colorized text to stderr, final agent message to stdout
   - **JSONL** (`event_processor_with_jsonl_output.rs`): structured `ThreadEvent` objects to stdout, one per line

4. **Wire types** (`exec_events.rs`): `ThreadEvent` tagged enum with variants for agent messages, command execution, file changes, MCP tool calls, web search, errors, etc.

### exec vs tui

- **exec**: headless, stdin/stdout, for automation. Prompt goes in, answer comes out.
- **tui**: interactive fullscreen terminal UI with ratatui. For human use.

Both consume the same agent engine via `InProcessAppServerClient`.

## Key Considerations

- `#![deny(clippy::print_stdout)]` is enforced. In default mode, stdout is reserved exclusively for the final agent message. In JSONL mode, stdout is reserved for JSONL events. All other output goes to stderr.
- The `Cli` struct in `cli.rs` is embedded into the main `MultitoolCli` in `codex-rs/cli/` as the `exec` subcommand.
- Integration tests in `tests/suite/` cover sandbox, resume, auth, patches, MCP, output schema, and more. Tests use `wiremock` for HTTP mocking and follow the `tests/all.rs` -> `tests/suite/mod.rs` pattern.
- The binary name is `orbit-code-exec`, library name is `orbit_code_exec`.
