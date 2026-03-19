# codex-rs/core/src/bin/

Binary targets defined by the `codex-core` crate.

## What this folder does

Contains the source for auxiliary binaries that are built alongside the `codex-core` library.

## Key files

| File | Purpose |
|------|---------|
| `config_schema.rs` | `codex-write-config-schema` binary: generates the JSON Schema for `config.toml` and writes it to `config.schema.json`. Uses `codex_core::config::schema::write_config_schema()`. |

## Usage

```bash
# Generate/update config.schema.json at the crate root
cargo run --bin codex-write-config-schema

# Write to a custom path
cargo run --bin codex-write-config-schema -- --out /path/to/output.json
```

## Imports from

- `codex_core::config::schema` -- `write_config_schema()` function
- `clap` -- CLI argument parsing

## Exports to

- Produces `codex-rs/core/config.schema.json` (checked into the repository)
