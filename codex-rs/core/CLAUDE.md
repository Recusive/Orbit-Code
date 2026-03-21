# codex-rs/core/

The central business-logic engine. Every consumer (TUI, exec, app-server, CLI) depends on this crate as a library.

## Build & Test

```bash
cargo build -p orbit-code-core
cargo test -p orbit-code-core            # all unit + integration tests
cargo test -p orbit-code-core -- suite::tools  # run one test module
just write-config-schema                 # after ANY change to ConfigToml or config types
```

The crate also produces a binary `orbit-code-write-config-schema` (declared in Cargo.toml) that regenerates `config.schema.json`. Always run `just write-config-schema` after touching anything under `src/config/`.

## Architecture

### Session hierarchy

Three layers, each wrapping the one below:

1. **`ThreadManager`** (`thread_manager.rs`) -- manages multiple concurrent agent threads. Owns shared resources (auth, MCP servers, plugins, skills, models, file watcher). The entry point consumers call to create threads.
2. **`CodexThread`** (`orbit_code_thread.rs`) -- public wrapper around a single session. Exposes `submit(Op)`, receives `EventMsg` via channel. This is what TUI/app-server hold onto.
3. **`Session` (aka `Codex`)** (`codex.rs`) -- the inner agent loop. Orchestrates turns: sends prompts to the model, processes streaming responses, dispatches tool calls, handles approvals, emits events.

### Op/EventMsg data flow

Communication is unidirectional message passing:

- **`Op`** (defined in `orbit-code-protocol`) -- operations submitted *into* the session: `UserTurn`, `ApprovalResponse`, `Abort`, `SteerInput`, config changes, etc.
- **`EventMsg`** (defined in `orbit-code-protocol`) -- events emitted *out of* the session: `TurnStarted`, `AgentMessage`, `ToolCall`, `ToolResult`, `TurnCompleted`, errors, etc.

Consumers call `codex_thread.submit(Op::UserTurn { ... })` and read events from the `EventMsg` receiver channel. The session never calls back into the consumer directly.

### Tool execution pipeline

```
Model emits function_call
  -> ToolRouter (registry lookup + dispatch by tool name)
    -> ToolOrchestrator (approval check -> sandbox selection -> execute -> retry on denial)
      -> ToolRuntime (platform-specific: shell, apply_patch, file ops, MCP, etc.)
        -> SandboxManager (wraps command in Seatbelt/seccomp/landlock/Windows restricted token)
          -> exec (process spawn, output capture, truncation)
```

Key modules: `tools/router.rs` (dispatch), `tools/orchestrator.rs` (approval+sandbox+retry), `tools/registry.rs` (registration), `tools/handlers/` (per-tool implementations), `tools/runtimes/` (platform backends).

### Config layering

Config is assembled from multiple sources (system -> user -> project -> runtime), merged by `config_loader/`. The `Config` struct (`config/mod.rs`) is the resolved result. Config types derive `JsonSchema`; the schema binary writes `config.schema.json`.

### Auth manager

`AuthManager` (`auth/manager.rs`) is the session-level auth cache. It holds provider-filtered credentials, handles token refresh, and coordinates 401 recovery (`auth/recovery.rs`). Storage backends (file, keyring, ephemeral) live in `auth/storage.rs`. Multi-provider persistence with merge-on-save lives in `auth/persistence.rs`.

## Key Considerations

- **Mirror tui/tui_app_server**: Any change to TUI-facing behavior in core must be reflected in both `tui/` and `tui_app_server/` crates. This is a root CLAUDE.md convention (rule 54).
- **Test conventions**: Integration tests use `TestCodexBuilder` (in `tests/common/test_codex.rs`) with fluent `.with_config()`, `.with_model()`, `.with_auth()` chains. HTTP mocking uses `wiremock::MockServer` + helpers from `core_test_support::responses`. Use `mount_sse_once` for mock API responses, hold onto the returned `ResponseMock` to assert outbound requests.
- **Single test binary**: All integration tests funnel through `tests/all.rs` -> `tests/suite/mod.rs`. Never create additional top-level test files.
- **Sibling `*_tests.rs` files**: Unit tests live in `src/foo_tests.rs` next to `src/foo.rs`, not inline. This is consistent throughout the crate.
- **Schema regeneration**: After changing `ConfigToml`, config types, or anything under `src/config/`, run `just write-config-schema`. After app-server protocol changes, run `just write-app-server-schema`.
- **Sandbox env vars**: Never modify code related to `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` or `CODEX_SANDBOX_ENV_VAR`. These are set by the sandbox runtime and existing guards are intentional.
- **No process env mutation in tests**: Pass environment-derived flags as parameters, don't set env vars in test code.
- **`include_str!` / `include_bytes!`**: If you add any, update the crate's `BUILD.bazel` (`compile_data`) or Bazel CI will fail even when Cargo passes.
