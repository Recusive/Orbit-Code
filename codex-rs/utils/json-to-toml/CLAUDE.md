# codex-rs/utils/json-to-toml/

Single recursive function converting `serde_json::Value` to `toml::Value`. Used for merging JSON-sourced config overrides into the TOML config tree.

## Build & Test
```bash
cargo build -p orbit-code-utils-json-to-toml
cargo test -p orbit-code-utils-json-to-toml
```
