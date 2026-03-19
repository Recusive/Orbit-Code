# codex-rs/core/src/config/

This file applies to `codex-rs/core/src/config/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Configuration types, builder, schema generation, and runtime config management for Codex.

### What this folder does

This is the largest module in `codex-core`. It defines the `Config` struct (the fully resolved runtime configuration) and the `ConfigBuilder` that constructs it from layered TOML sources. Key responsibilities:

- **Config struct**: Holds every runtime setting -- model selection, sandbox policy, approval policy, MCP servers, features, personality, shell environment, tools, permissions, and more.
- **ConfigBuilder**: Merges layered config (system, user, project, CLI flags) into a final `Config` via `ConfigToml` deserialization, applying defaults, validation, and constraint enforcement.
- **Schema generation** (`schema.rs`): Produces a JSON Schema for `config.toml` using `schemars`, including custom schemas for features and MCP servers.
- **Agent roles** (`agent_roles.rs`): Parsing and resolution of agent role configuration files.
- **Profiles** (`profile.rs`): Named configuration profiles that group model/provider/settings.
- **Types** (`types.rs`): All TOML-serializable config types (`McpServerConfig`, `OtelConfig`, `MemoriesConfig`, `ShellEnvironmentPolicy`, `SkillsConfig`, etc.).
- **Permissions** (`permissions.rs`): Sandbox permission configuration and policy resolution.
- **Config editing** (`edit.rs`): Programmatic config.toml modification (add/remove projects, trust levels).
- **Network proxy** (`network_proxy_spec.rs`): Managed network proxy configuration.
- **Managed features** (`managed_features.rs`): Enterprise/managed feature constraint handling.
- **Service layer** (`service.rs`): Config service for live-reload and change notifications.

### Key files

| File | Purpose |
|------|---------|
| `mod.rs` | `Config` struct, `ConfigBuilder`, `ConfigOverrides`, `ConfigToml`, main build logic |
| `types.rs` | All TOML-serializable config structs and enums |
| `schema.rs` | JSON Schema generation for `config.toml` |
| `permissions.rs` | Sandbox policy resolution from config |
| `profile.rs` | Named config profiles |
| `agent_roles.rs` | Agent role config parsing |
| `edit.rs` | Programmatic config file editing |
| `service.rs` | Config live-reload service |
| `managed_features.rs` | Enterprise managed feature constraints |
| `network_proxy_spec.rs` | Network proxy configuration |

### Imports from

- `codex_config` -- Lower-level config parsing, layer merging, requirement types
- `codex_protocol` -- `SandboxPolicy`, `AskForApproval`, `SandboxMode`, config types
- `crate::config_loader` -- Config layer stack loading
- `crate::features` -- Feature flags integration
- `crate::model_provider_info` -- Model provider metadata

### Exports to

- Used by virtually every module in `codex-core` via `crate::config::Config`
- Re-exported publicly for `codex-tui`, `codex-exec`, `codex-app-server`
