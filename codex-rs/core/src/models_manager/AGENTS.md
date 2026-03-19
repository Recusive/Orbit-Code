# codex-rs/core/src/models_manager/

This file applies to `codex-rs/core/src/models_manager/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Model metadata management, caching, collaboration mode presets, and version tracking.

### What this folder does

Manages the registry of available AI models, their capabilities and metadata, collaboration mode presets, and model caching. This is the layer between raw model provider info and the session's model selection logic.

Key responsibilities:
- **ModelsManager** (`manager.rs`): Fetches and caches model metadata from remote APIs, provides model info lookups, handles model availability and refresh strategies.
- **Model info** (`model_info.rs`): Structures for model capabilities (context window, input modalities, supported features).
- **Collaboration mode presets** (`collaboration_mode_presets.rs`): Predefined collaboration mode configurations (default, pair programming, plan, execute).
- **Model presets** (`model_presets.rs`): Default model configurations and presets.
- **Cache** (`cache.rs`): Local filesystem caching for model metadata with ETag-based invalidation.

### Key files

| File | Purpose |
|------|---------|
| `mod.rs` | Module declarations; `client_version_to_whole()` helper |
| `manager.rs` | `ModelsManager` -- model metadata fetching, caching, refresh logic |
| `model_info.rs` | Model capability and metadata structs |
| `collaboration_mode_presets.rs` | Collaboration mode preset definitions |
| `model_presets.rs` | Default model configuration presets |
| `cache.rs` | Filesystem-based model metadata cache |

### Imports from

- `crate::config` -- `Config` for model provider settings
- `crate::model_provider_info` -- Provider metadata and wire API types
- `crate::auth` -- `AuthManager` for authenticated API requests

### Exports to

- `crate::codex` -- `ModelsManager` held in `SessionServices`
- `crate::tasks` -- model info used for turn configuration
- `crate::client` -- model metadata for API request construction
