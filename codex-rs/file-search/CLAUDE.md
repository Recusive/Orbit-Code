# codex-rs/file-search/

Fuzzy file search engine using `nucleo` for scoring and `ignore` for gitignore-aware filesystem traversal. Provides both a library API and a standalone CLI binary.

## Build & Test
```bash
cargo build -p orbit-code-file-search
cargo test -p orbit-code-file-search
```

## Architecture

The search engine uses a two-thread architecture: a walker thread discovers files via the `ignore` crate (same walker as ripgrep), and a matcher thread scores them with the `nucleo` fuzzy matcher (same engine as Helix editor). Threads coordinate via `crossbeam-channel`. Two modes are supported: one-shot search via `run()` returning `FileSearchResults`, and interactive sessions via `create_session()` that allow live query updates and streaming results through a `SessionReporter` trait without re-walking the filesystem.

## Key Considerations
- Walker uses `require_git(true)` to scope gitignore rules to actual git repositories only
- Nucleo is configured with `Config::DEFAULT.match_paths()` for path-aware scoring
- Walker and matcher run on OS threads (not Tokio tasks) -- this is intentional for CPU-bound fuzzy matching
- Cancellation is cooperative via `Arc<AtomicBool>` checked periodically in both threads
- The `FileSearchOptions` struct controls limits, excludes, thread count, gitignore behavior, and index computation
