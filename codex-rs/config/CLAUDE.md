# codex-rs/config/

Layered configuration loading, merging, validation, and constraint enforcement for Orbit Code.

## Build & Test

```bash
cargo test -p orbit-code-config        # Run tests
just fmt                               # Format after changes
just fix -p orbit-code-config          # Clippy
just write-config-schema               # Regenerate JSON schema after ConfigToml changes
```

## Architecture

Configuration flows through a layered stack with precedence (later layers override earlier):

1. **System/managed config** -- enterprise/MDM-pushed configuration
2. **User config** -- `~/.orbit-code/config.toml`
3. **Project config** -- `.orbit-code/config.toml` in the project root
4. **CLI overrides** -- command-line flags converted to a TOML layer

Core data flow:
- `ConfigLayerStack` holds an ordered list of `ConfigLayerEntry` values (each wrapping a `toml::Value` with a fingerprint and source metadata)
- `merge_toml_values()` deep-merges TOML tables (recursive table merge, scalar overwrite)
- The merged result is deserialized into `ConfigRequirementsToml` -> `ConfigRequirements`
- `Constrained<T>` wraps values that may be locked by requirements (e.g., sandbox mode forced by enterprise policy). Attempting to change a constrained value produces a `ConstraintError`
- `version_for_toml()` computes SHA fingerprints for change detection
- `CloudRequirementsLoader` fetches remote/managed requirements asynchronously

Key modules:
- `state.rs` -- `ConfigLayerStack`, `ConfigLayerEntry`, layer ordering and merge
- `config_requirements.rs` -- all requirement types (`ConfigRequirementsToml`, network constraints, sandbox mode, MCP server requirements, etc.)
- `constraint.rs` -- `Constrained<T>` and constraint enforcement
- `diagnostics.rs` -- error types with TOML source positions for user-facing messages
- `overrides.rs` -- `build_cli_overrides_layer()` for CLI flag -> TOML conversion

## Key Considerations

- Config types must derive `JsonSchema`. After any change to `ConfigToml` or related types, run `just write-config-schema` to regenerate `core/config.schema.json`.
- This crate is a library only (no binary). It is consumed primarily by `orbit-code-core` for session configuration and by `orbit-code-app-server` to expose config layers to IDE clients.
- `RequirementsExecPolicy` in `requirements_exec_policy.rs` defines execution policy rules (prefix patterns, allow/deny) that are separate from `orbit-code-execpolicy` -- these come from managed config requirements.
- No integration tests directory -- all tests are unit tests within the source modules.
