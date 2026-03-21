# codex-rs/utils/rustls-provider/src/

One-time process-wide rustls crypto provider initialization using `ring`. Necessary because rustls cannot auto-select a provider when both `ring` and `aws-lc-rs` are in the dependency graph.

## Build & Test
```bash
cargo build -p orbit-code-utils-rustls-provider
cargo test -p orbit-code-utils-rustls-provider
```
