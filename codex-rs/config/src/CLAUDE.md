# codex-rs/config/src/

Source for the `orbit-code-config` crate (library only).

Layered config system: `state.rs` (ConfigLayerStack, merge), `config_requirements.rs` (all requirement types), `constraint.rs` (Constrained<T>), `diagnostics.rs` (errors with TOML source positions), `merge.rs` (deep TOML merge), `overrides.rs` (CLI flags to TOML layer), `fingerprint.rs` (SHA change detection), `cloud_requirements.rs` (remote config loading), `requirements_exec_policy.rs` (managed exec policy rules).
