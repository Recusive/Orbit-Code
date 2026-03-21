# codex-rs/utils/cli/

Shared clap-derived CLI argument types for `--approval-mode`, `--sandbox` (`-s`), and `-c key=value` config overrides. Reused by TUI, exec, and app-server entry points.

## Build & Test
```bash
cargo build -p orbit-code-utils-cli
cargo test -p orbit-code-utils-cli
```

## Key Considerations
- `CliConfigOverrides::apply_on_value()` applies dotted-path overrides onto a `toml::Value` tree and canonicalizes legacy key aliases.
