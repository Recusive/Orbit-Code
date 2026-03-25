# codex-rs/core/src/config_loader/

This file applies to `codex-rs/core/src/config_loader/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Layered configuration loading from multiple sources (system, user, project, managed, runtime).

### What this folder does

Implements the multi-layer config loading pipeline that builds a `ConfigLayerStack` from various sources. The layers are merged in precedence order:

1. **Cloud requirements** -- Managed cloud constraints (highest precedence for requirements)
2. **Admin/MDM** -- macOS managed device profiles (via `macos.rs`)
3. **System** -- `/etc/codex/config.toml` (Unix) or `%ProgramData%\OpenAI\Codex\config.toml` (Windows)
4. **User** -- `$ORBIT_HOME/config.toml`
5. **Project** -- `.orbit/config.toml` files from cwd up to project root (trust-gated)
6. **Runtime** -- CLI flags, UI overrides (lowest layer, highest precedence for config)

Key features:
- **Project trust**: Project-level configs require explicit trust in the user config before being applied. Uses git root detection for trust key resolution.
- **Requirements vs config**: Requirements (`requirements.toml`) constrain allowed values; config (`config.toml`) sets actual values.
- **Path resolution**: All relative paths in config files are resolved against their containing directory.
- **Legacy support**: Maps deprecated `managed_config.toml` fields to the modern requirements system.

### Key files

| File | Purpose |
|------|---------|
| `mod.rs` | `load_config_layers_state()` -- main entry point; project trust logic; path resolution |
| `layer_io.rs` | Low-level file I/O for loading managed_config.toml layers |
| `macos.rs` | macOS-specific managed admin requirements loading via MDM profiles |
| `tests.rs` | Integration tests for config loading |
| `README.md` | Documentation for the config loading system |

### Imports from

- `codex_config` -- `ConfigLayerStack`, `ConfigLayerEntry`, `ConfigRequirements`, merge utilities
- `crate::config::ConfigToml` -- The deserialization target for config files
- `crate::git_info` -- Git root detection for project trust resolution

### Exports to

- `crate::config::Config` -- The loaded `ConfigLayerStack` feeds into `ConfigBuilder`
- Re-exported types: `ConfigLayerStack`, `ConfigLayerEntry`, `ConfigRequirements`, `LoaderOverrides`, etc.
