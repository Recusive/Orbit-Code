# codex-rs/hooks/

Hook system for lifecycle events in the Orbit Code agent. Hooks are external commands that receive JSON on stdin and return JSON on stdout.

## Build & Test

```bash
cargo test -p orbit-code-hooks            # Run tests
just fmt                                   # Format after changes
just fix -p orbit-code-hooks               # Clippy
just write-hooks-schema                    # Regenerate JSON schema fixtures after wire type changes
```

## Architecture

### Hook lifecycle

Hooks fire at three points in the agent session:

1. **SessionStart** -- when a session begins. Can inject context (system messages) or block the session.
2. **UserPromptSubmit** -- when the user submits a prompt. Can modify the prompt, inject context, or block submission.
3. **Stop** -- when the agent session ends. Can perform cleanup or emit final context.

### Data flow

```
ConfigLayerStack
    |
    v
discovery.rs -- scans config layers for hook definitions
    |
    v
ConfiguredHandler -- (event_name, command, matcher, timeout, source_path)
    |
    v
ClaudeHooksEngine -- holds handlers, dispatches per event type
    |
    v
command_runner.rs -- spawns hook command, sends JSON stdin, reads JSON stdout, enforces timeout
    |
    v
output_parser.rs -- parses JSON output into typed outcomes (continue/block, system messages, etc.)
```

### Key abstractions

- `Hooks` (registry.rs) -- public API. Created from `HooksConfig`. Exposes `run_session_start()`, `run_user_prompt_submit()`, `run_stop()`, and `preview_*` methods. Also dispatches legacy notify hooks.
- `ClaudeHooksEngine` (engine/mod.rs) -- internal engine that holds discovered `ConfiguredHandler` entries and delegates to per-event run/preview functions.
- `events/` -- per-event modules (`session_start.rs`, `user_prompt_submit.rs`, `stop.rs`) with typed `*Request` and `*Outcome` structs.
- `schema.rs` -- JSON Schema wire types (`*CommandInput`, `*OutputWire`) and `write_schema_fixtures()` for generating schema files.

### Legacy hooks

`legacy_notify.rs` provides backward-compatible fire-and-forget notification hooks (no JSON protocol, just spawns a command with JSON as an argument).

## Key Considerations

- After changing any wire type in `schema.rs` or the event input/output structs, run `just write-hooks-schema` to regenerate the fixtures in `schema/generated/`.
- Hook commands run as child processes with a configurable timeout. The shell used is controlled by `HooksConfig.shell_program` and `shell_args`.
- Hooks are discovered from the `ConfigLayerStack` (same layered config as the rest of the system). Each config layer can define hooks, and they are merged by the discovery module.
- This is a library crate with one binary (`write_hooks_schema_fixtures` in `src/bin/`) used only for schema generation.
- No `tests/` directory -- tests are unit tests within source modules and the engine submodules.
