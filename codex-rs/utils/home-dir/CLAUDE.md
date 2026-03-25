# codex-rs/utils/home-dir/

Locates the Orbit Code configuration home directory. Honors `ORBIT_HOME` environment variable (must be an existing directory) and falls back to `~/.orbit`.

## Build & Test
```bash
cargo build -p orbit-code-utils-home-dir
cargo test -p orbit-code-utils-home-dir
```
