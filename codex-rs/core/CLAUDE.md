# codex-rs/core/

The `codex-core` crate is the central business-logic engine of the Codex CLI. It implements the AI agent loop, tool execution, configuration management, sandboxing, session persistence, and all supporting subsystems. Every consumer of Codex (TUI, headless exec, app-server, VS Code extension) depends on this crate as a library.

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

- **Upstream consumers**: `codex-tui`, `codex-exec`, `codex-app-server`, `codex-cli` all depend on `codex-core`.
- **Downstream workspace crates**: `codex-protocol`, `codex-config`, `codex-client`, `codex-connectors`, `codex-hooks`, `codex-secrets`, `codex-skills`, `codex-execpolicy`, `codex-state`, `codex-otel`, `codex-network-proxy`, `codex-rmcp-client`, and many utility crates.

## Key files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Crate manifest with ~60 workspace dependencies |
| `src/lib.rs` | Library root; declares all modules, re-exports public API |
| `src/codex.rs` | `Session` -- the core agent loop and turn orchestration |
| `src/codex_thread.rs` | `CodexThread` -- wraps a session for external consumers |
| `src/thread_manager.rs` | `ThreadManager` -- manages multiple concurrent threads |
| `src/config/mod.rs` | `Config` struct and builder; merges all config layers |
| `src/client.rs` | `ModelClient` -- HTTP client for the Responses API |
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
cargo build -p codex-core
cargo test -p codex-core
```

The `codex-write-config-schema` binary (declared in `Cargo.toml`) regenerates `config.schema.json`.
