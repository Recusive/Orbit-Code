# codex-rs/core/src/config_loader/

Layered configuration loading from multiple sources (system, user, project, managed, runtime).

## What this folder does

Implements the multi-layer config loading pipeline that builds a `ConfigLayerStack` from various sources. The layers are merged in precedence order:

1. **Cloud requirements** -- Managed cloud constraints (highest precedence for requirements)
2. **Admin/MDM** -- macOS managed device profiles (via `macos.rs`)
3. **System** -- `/etc/codex/config.toml` (Unix) or `%ProgramData%\OpenAI\Codex\config.toml` (Windows)
4. **User** -- `$CODEX_HOME/config.toml`
5. **Project** -- `.codex/config.toml` files from cwd up to project root (trust-gated)
6. **Runtime** -- CLI flags, UI overrides (lowest layer, highest precedence for config)

Key features:
- **Project trust**: Project-level configs require explicit trust in the user config before being applied. Uses git root detection for trust key resolution.
- **Requirements vs config**: Requirements (`requirements.toml`) constrain allowed values; config (`config.toml`) sets actual values.
- **Path resolution**: All relative paths in config files are resolved against their containing directory.
- **Legacy support**: Maps deprecated `managed_config.toml` fields to the modern requirements system.

## Key files

| File | Purpose |
|------|---------|
| `mod.rs` | `load_config_layers_state()` -- main entry point; project trust logic; path resolution |
| `layer_io.rs` | Low-level file I/O for loading managed_config.toml layers |
| `macos.rs` | macOS-specific managed admin requirements loading via MDM profiles |
| `tests.rs` | Integration tests for config loading |
| `README.md` | Documentation for the config loading system |

## Imports from

- `codex_config` -- `ConfigLayerStack`, `ConfigLayerEntry`, `ConfigRequirements`, merge utilities
- `crate::config::ConfigToml` -- The deserialization target for config files
- `crate::git_info` -- Git root detection for project trust resolution

## Exports to

- `crate::config::Config` -- The loaded `ConfigLayerStack` feeds into `ConfigBuilder`
- Re-exported types: `ConfigLayerStack`, `ConfigLayerEntry`, `ConfigRequirements`, `LoaderOverrides`, etc.
