# codex-rs/utils/cache/src/

Source directory for the `codex-utils-cache` crate.

## Key files

- `lib.rs` -- single-file implementation containing:
  - `BlockingLruCache<K, V>` -- wraps `lru::LruCache` in a `Tokio::Mutex`; all operations gracefully degrade to no-ops when no Tokio runtime is available (via `tokio::runtime::Handle::try_current`)
  - Methods: `new`, `try_with_capacity`, `get`, `insert`, `remove`, `clear`, `get_or_insert_with`, `get_or_try_insert_with`, `with_mut`, `blocking_lock`
  - `sha1_digest(bytes: &[u8]) -> [u8; 20]` -- SHA-1 hash helper
  - `lock_if_runtime` -- internal helper using `tokio::task::block_in_place` for safe blocking lock acquisition
  - Tests verifying store/retrieve, LRU eviction, and disabled behavior without a runtime
