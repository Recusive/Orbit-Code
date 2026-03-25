# codex-rs/utils/home-dir/src/

Locates the Codex configuration home directory. Honors `ORBIT_HOME` environment variable (must be an existing directory) and falls back to `~/.orbit`.

## Build & Test
```bash
cargo build -p orbit-code-utils-home-dir
cargo test -p orbit-code-utils-home-dir
```
