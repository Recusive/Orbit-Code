# codex-rs/core/

The `orbit-code-core` crate is the central business-logic engine of the Codex CLI. It implements the AI agent loop, tool execution, configuration management, sandboxing, session persistence, and all supporting subsystems. Every consumer of Codex (TUI, headless exec, app-server, VS Code extension) depends on this crate as a library.

## What this folder does

- Provides the `Session` abstraction that drives a single agent conversation (model calls, tool dispatch, approval flow, rollout recording).
- Manages multi-agent orchestration (spawning sub-agents, role configuration, agent lifecycle guards).
- Loads and merges layered configuration from system, user, project, and runtime sources.
- Enforces sandboxing policies (macOS Seatbelt, Linux seccomp/landlock, Windows restricted tokens).
- Connects to MCP servers and routes tool calls through a unified registry/router.
- Handles authentication (API keys, OAuth tokens, keyring/file/ephemeral storage).
- Persists session rollouts and provides resume/fork capabilities.
- Implements the memory subsystem (extraction, consolidation, memory tool).
- Manages context windows, compaction, and token accounting.

## Where it plugs into

- **Upstream consumers**: `orbit-code-tui`, `orbit-code-exec`, `orbit-code-app-server`, `orbit-code` (cli) all depend on `orbit-code-core`.
- **Downstream workspace crates**: `orbit-code-protocol`, `orbit-code-config`, `orbit-code-client`, `orbit-code-hooks`, `orbit-code-secrets`, `orbit-code-skills`, `orbit-code-execpolicy`, `orbit-code-state`, `orbit-code-otel`, `orbit-code-network-proxy`, `orbit-code-rmcp-client`, and many utility crates.

## Key files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Crate manifest with ~60 workspace dependencies |
| `src/lib.rs` | Library root; declares all modules, re-exports public API |
| `src/codex.rs` | `Session` -- the core agent loop and turn orchestration |
| `src/orbit_code_thread.rs` | `CodexThread` -- wraps a session for external consumers |
| `src/thread_manager.rs` | `ThreadManager` -- manages multiple concurrent threads |
| `src/config/mod.rs` | `Config` struct and builder; merges all config layers |
| `src/client.rs` | `ModelClient` -- HTTP client for the Responses API |
| `src/auth.rs` | Auth types (`CodexAuth`, `AuthMode`), constants |
| `src/auth/manager.rs` | `AuthManager` -- session auth cache, provider-filtered lookups |
| `src/auth/persistence.rs` | Save/load auth (v2 multi-provider format, merge-on-save) |
| `src/auth/recovery.rs` | `UnauthorizedRecovery` -- 401 recovery state machine |
| `src/auth/storage.rs` | Storage backends (file, keyring, ephemeral), `ProviderName` |
| `src/anthropic_auth/` | Anthropic OAuth types, token refresh, request modifications |
| `src/tools/mod.rs` | Tool formatting and execution output handling |
| `src/tools/router.rs` | `ToolRouter` -- dispatches tool calls to handlers |
| `src/sandboxing/mod.rs` | `SandboxManager` -- transforms commands for sandboxed execution |
| `src/rollout/mod.rs` | Session rollout persistence and discovery |
| `src/features.rs` | Centralized feature flags (`Feature` enum, `Features` container) |
| `config.schema.json` | Generated JSON Schema for `config.toml` |
| `models.json` | Built-in model metadata registry |
| `prompt.md` | Default system prompt template |

## Build

```bash
cargo build -p orbit-code-core
cargo test -p orbit-code-core
```

The `orbit-code-write-config-schema` binary (declared in `Cargo.toml`) regenerates `config.schema.json`.
