# codex-rs/exec/

This file applies to `codex-rs/exec/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-exec` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-exec`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Headless (non-interactive) CLI for running the Codex agent. This is the `codex-exec` binary, designed for automation, CI pipelines, and scripted usage where no TUI is needed.

### What this folder does

Provides the `codex-exec` binary and its library crate (`codex_exec`). It accepts a prompt (via CLI argument or stdin), sends it to the Codex agent engine, processes streaming events, and writes the final agent response to stdout or a file. Supports two output modes: human-readable (colorized stderr transcript + final message on stdout) and JSONL (structured `ThreadEvent` objects on stdout).

### What it plugs into

- **codex-core** -- the agent engine that drives model interactions, tool execution, and session management
- **codex-app-server-client** -- in-process app-server client used to communicate with the agent via JSON-RPC request/response
- **codex-app-server-protocol** -- typed JSON-RPC message definitions (ClientRequest, ServerNotification, etc.)
- **codex-protocol** -- core event types (Event, EventMsg, SessionConfiguredEvent, SandboxPolicy, etc.)
- **codex-arg0** -- multi-binary dispatch (the same executable can also run as `codex-linux-sandbox`)
- **codex-otel** -- OpenTelemetry tracing integration
- **codex-feedback** -- feedback collection

### Imports from

- `codex-core`: Config, ConfigBuilder, AuthManager, session management, exec policy checking, OSS provider utilities
- `codex-app-server-client`: InProcessAppServerClient, InProcessClientStartArgs, InProcessServerEvent
- `codex-app-server-protocol`: ClientRequest variants (ThreadStart, TurnStart, ReviewStart, etc.), ServerRequest variants, RequestId
- `codex-protocol`: Event, EventMsg, AskForApproval, SandboxMode, SessionSource, ReviewTarget, UserInput
- `codex-arg0`: Arg0DispatchPaths, arg0_dispatch_or_else

### Exports to

- Used as the `codex-exec` binary entry point (via `codex-cli` arg0 dispatch or standalone)
- Library exports: `Cli`, `Command`, `ReviewArgs`, `run_main()`, `exec_events` module, `event_processor_with_jsonl_output` module

### Main functions / purpose

- `run_main(cli, arg0_paths)` -- top-level async entry point: loads config, initializes OTEL, starts an in-process app-server client, sends the initial prompt/review request, then enters the event loop
- `run_exec_session(args)` -- the core event loop: creates an EventProcessor (human or JSON), starts a thread, sends a turn, and processes events until TurnComplete or shutdown
- `resolve_prompt(prompt_arg)` -- reads prompt from CLI arg or stdin (with UTF-8/16 BOM handling)
- `build_review_request(args)` -- constructs a ReviewRequest from CLI flags (--uncommitted, --base, --commit, or custom prompt)

### Key files

- `Cargo.toml` -- crate metadata; binary is `codex-exec`, library is `codex_exec`
- `src/main.rs` -- binary entry point with arg0 dispatch (can run as codex-linux-sandbox)
- `src/lib.rs` -- main library: `run_main()`, event loop, session management, prompt resolution, review request building
- `src/cli.rs` -- clap CLI definition: `Cli` struct, `Command` enum (Resume, Review), `ResumeArgs`, `ReviewArgs`
- `src/event_processor.rs` -- `EventProcessor` trait and `CodexStatus` enum (Running, InitiateShutdown, Shutdown)
- `src/event_processor_with_human_output.rs` -- colorized stderr output implementation of EventProcessor
- `src/event_processor_with_jsonl_output.rs` -- JSONL stdout output implementation; converts internal `Event` stream to `ThreadEvent` JSONL
- `src/exec_events.rs` -- public types for the JSONL output format: `ThreadEvent`, `ThreadItem`, `ThreadItemDetails`, `Usage`, etc.
