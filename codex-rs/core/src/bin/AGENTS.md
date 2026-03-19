# codex-rs/core/src/bin/

This file applies to `codex-rs/core/src/bin/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Binary targets defined by the `codex-core` crate.

### What this folder does

Contains the source for auxiliary binaries that are built alongside the `codex-core` library.

### Key files

| File | Purpose |
|------|---------|
| `config_schema.rs` | `codex-write-config-schema` binary: generates the JSON Schema for `config.toml` and writes it to `config.schema.json`. Uses `codex_core::config::schema::write_config_schema()`. |

### Usage

```bash
# Generate/update config.schema.json at the crate root
cargo run --bin codex-write-config-schema

# Write to a custom path
cargo run --bin codex-write-config-schema -- --out /path/to/output.json
```

### Imports from

- `codex_core::config::schema` -- `write_config_schema()` function
- `clap` -- CLI argument parsing

### Exports to

- Produces `codex-rs/core/config.schema.json` (checked into the repository)
