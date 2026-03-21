# codex-rs/file-search/src/

Source for the `orbit-code-file-search` crate -- fuzzy file search with walker/matcher thread architecture.

## Module Layout
- **Core engine** (`lib.rs`): `run()` for one-shot search, `create_session()` for interactive sessions; walker thread (`walker_worker`), matcher thread (`matcher_worker`); types: `FileMatch`, `MatchType`, `FileSearchResults`, `FileSearchSnapshot`, `FileSearchOptions`, `SessionReporter`, `Reporter`
- **CLI** (`cli.rs`, `main.rs`): Clap `Cli` definition with pattern, limit, cwd, threads, exclude, json, compute-indices flags; `StdioReporter` for plain text, JSON, and ANSI-highlighted output
