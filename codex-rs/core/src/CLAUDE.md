# codex-rs/core/src/

Root source directory for the `orbit-code-core` library crate. Contains all Rust modules that implement the Codex agent engine.

## What this folder does

This is the main source tree for `orbit-code-core`. The entry point is `lib.rs`, which declares every module and re-exports the public API consumed by upstream crates (`orbit-code-tui`, `orbit-code-exec`, `orbit-code-app-server`).

## Module organization

The source tree is a mix of single-file modules and directory modules:

### Core session and agent loop
- `codex.rs` / `codex/` -- `Session` struct: the main agent loop, turn orchestration, event emission
- `orbit_code_thread.rs` -- `CodexThread`: public wrapper around a session for external consumers
- `orbit_code_delegate.rs` -- Delegate trait for session lifecycle callbacks
- `thread_manager.rs` -- `ThreadManager`: manages multiple concurrent agent threads

### Configuration
- `config/` -- `Config` struct, builder, schema, types, permissions, profiles, agent roles
- `config_loader/` -- Layered config loading from system/user/project/runtime sources
- `features.rs` / `features/` -- Feature flags (`Feature` enum, `Features` container, legacy aliases)

### Tools and execution
- `tools/` -- Tool registry, router, handlers, runtimes, sandboxing, orchestrator
- `unified_exec/` -- Interactive PTY process management with approvals and sandboxing
- `exec.rs` -- Low-level command execution and output capture
- `exec_env.rs` -- Environment variable setup for child processes
- `exec_policy.rs` -- Execution policy loading and validation
- `sandboxing/` -- Platform sandbox wrappers (Seatbelt, seccomp, landlock)

### Agent and multi-agent
- `agent/` -- Agent control, guards, roles, status tracking
- `tasks/` -- Session task types (regular, review, compact, undo, ghost snapshot, user shell)
- `state/` -- Session services, session state, turn state, active turn tracking

### External integrations
- `mcp/` -- MCP server management, tool collection, auth
- `mcp_connection_manager.rs` -- MCP connection lifecycle
- `mcp_tool_call.rs` -- MCP tool call execution
- `client.rs` -- HTTP client for the OpenAI Responses API
- `connectors.rs` -- App connectors integration

### Persistence and history
- `rollout/` -- Session rollout file persistence, discovery, listing, indexing
- `context_manager/` -- Conversation history management and token accounting
- `memories/` -- Memory extraction and consolidation pipeline
- `compact.rs` -- Context compaction logic
- `message_history.rs` -- Message history serialization

### Auth and security
- `auth.rs` -- Auth types (`CodexAuth`, `AuthMode`), constants, re-exports
- `auth/manager.rs` -- `AuthManager`: session auth cache, provider-filtered lookups, token refresh
- `auth/persistence.rs` -- Save/load auth to disk (v2 format, provider merging)
- `auth/recovery.rs` -- `UnauthorizedRecovery`: 401 recovery state machine
- `auth/storage.rs` -- Storage backends (file, keyring, ephemeral), `AuthDotJsonV2`, `ProviderName`
- `anthropic_auth/` -- Anthropic-specific: OAuth types, token refresh, request modifications (tool prefixing, system prompt)
- `guardian/` -- Guardian review for automated approval decisions
- `safety.rs` -- Platform sandbox detection
- `seatbelt.rs` -- macOS Seatbelt sandbox profile generation
- `landlock.rs` -- Linux landlock sandbox support

### Skills, plugins, instructions
- `skills/` -- Skill loading, rendering, injection, invocation
- `plugins/` -- Plugin discovery, marketplace, manifest loading
- `instructions/` -- User instruction loading and injection (AGENTS.md)
- `apps/` -- App connector instruction rendering

### Utilities
- `truncate.rs` -- Text and output truncation for model consumption
- `text_encoding.rs` -- Character encoding detection and conversion
- `path_utils.rs` -- Path normalization helpers
- `util.rs` -- General-purpose utility functions
- `git_info.rs` -- Git repository detection and metadata
- `shell.rs` -- Shell detection and configuration
- `terminal.rs` -- Terminal capability detection

## Key imports

- `orbit_code_protocol` -- Core protocol types (`Op`, `EventMsg`, `ResponseItem`, etc.)
- `orbit_code_config` -- Configuration TOML parsing and layer merging
- `orbit_code_client` -- HTTP API client
- `orbit_code_hooks` -- Hook execution engine
- `orbit_code_otel` -- OpenTelemetry instrumentation

## Key exports

- `Session`, `CodexThread`, `ThreadManager` -- Main consumer-facing types
- `Config`, `ConfigBuilder` -- Configuration management
- `AuthManager`, `CodexAuth` -- Authentication
- `ModelClient`, `ModelClientSession` -- API client
- `RolloutRecorder` -- Session persistence
- `Feature`, `Features` -- Feature flags
