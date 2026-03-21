# codex-rs/utils/cache/src/

Thread-safe LRU cache (`BlockingLruCache`) that gracefully degrades to a no-op when no Tokio runtime is present, plus a `sha1_digest` helper for content-based cache keys.

## Build & Test
```bash
cargo build -p orbit-code-utils-cache
cargo test -p orbit-code-utils-cache
```

## Key Considerations
- All cache operations are no-ops when called outside a Tokio runtime (uses `Handle::try_current` to detect).
- Uses `tokio::task::block_in_place` for lock acquisition, so callers must be on a multi-threaded runtime.
