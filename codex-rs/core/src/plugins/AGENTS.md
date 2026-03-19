# codex-rs/core/src/plugins/

This file applies to `codex-rs/core/src/plugins/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Plugin discovery, installation, marketplace integration, and lifecycle management.

### What this folder does

Implements the full plugin ecosystem for Codex, enabling extensibility through installable plugins that can provide MCP servers, app connectors, skills, and custom instructions.

Key responsibilities:
- **PluginsManager** (`manager.rs`): Central manager for plugin discovery, loading, installation, and uninstallation. Maintains the mapping between plugins and their capabilities (MCP servers, app connectors).
- **Marketplace** (`marketplace.rs`): Integration with plugin marketplaces for browsing, searching, and installing plugins. Supports auth policies and install policies.
- **Manifest** (`manifest.rs`): Plugin manifest parsing (`PluginManifestInterface`) -- reads plugin metadata, capabilities, and configuration from manifest files.
- **Store** (`store.rs`): Local plugin storage on disk, managing installed plugin directories under `$CODEX_HOME/plugins/`.
- **Discovery** (`discoverable.rs`): Lists plugins that can be suggested to users via the `tool_suggest` tool.
- **Injection** (`injection.rs`): Builds plugin instruction injections for the model context.
- **Rendering** (`render.rs`): Renders plugin sections for system prompts.
- **Curated repo** (`curated_repo.rs`): Syncs the OpenAI curated plugins repository for trusted plugin sources.
- **Remote** (`remote.rs`): Fetches remote plugin metadata and featured plugin IDs.
- **Toggles** (`toggles.rs`): Plugin enable/disable state management.

### Key files

| File | Purpose |
|------|---------|
| `mod.rs` | Module declarations and public re-exports |
| `manager.rs` | `PluginsManager` -- the main plugin lifecycle coordinator |
| `marketplace.rs` | Marketplace browsing and installation |
| `manifest.rs` | Plugin manifest parsing and validation |
| `store.rs` | Local filesystem plugin store |
| `discoverable.rs` | Plugin discovery for tool suggestions |
| `injection.rs` | Plugin instruction injection into model context |
| `render.rs` | Plugin section rendering for prompts |
| `curated_repo.rs` | Curated plugin repository sync |
| `remote.rs` | Remote plugin metadata fetching |

### Imports from

- `crate::config` -- `Config`, `PluginConfig`
- `crate::skills` -- Skill integration for plugin-provided skills
- `crate::mcp` -- MCP server integration for plugin-provided servers

### Exports to

- `crate::mcp` -- `PluginsManager` provides plugin-sourced MCP servers
- `crate::codex` -- Plugin instructions injected during prompt construction
- `crate::state` -- `PluginsManager` held in `SessionServices`
- Public API for `codex-app-server` and `codex-tui` plugin management
