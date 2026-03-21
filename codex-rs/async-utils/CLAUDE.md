# codex-rs/async-utils/

Async utility extensions for Tokio. Provides `OrCancelExt` trait that lets any `Future` be raced against a `CancellationToken`, returning `Err(CancelErr::Cancelled)` if the token fires first.

## Build & Test
```bash
cargo build -p orbit-code-async-utils
cargo test -p orbit-code-async-utils
```
