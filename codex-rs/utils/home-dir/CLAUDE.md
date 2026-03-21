# codex-rs/utils/home-dir/

Locates the Codex configuration home directory. Honors `CODEX_HOME` environment variable (must be an existing directory) and falls back to `~/.codex`.

## Build & Test
```bash
cargo build -p orbit-code-utils-home-dir
cargo test -p orbit-code-utils-home-dir
```
