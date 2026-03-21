# codex-rs/environment/src/

Filesystem trait definition and local implementation.

## Module Layout

- **lib** (`lib.rs`) -- `Environment` struct with `get_filesystem()` accessor; module declarations and public re-exports
- **fs** (`fs.rs`) -- `ExecutorFileSystem` async trait (read, write, mkdir, metadata, readdir, remove, copy), `LocalFileSystem` implementation via `tokio::fs`, option types (`CreateDirectoryOptions`, `RemoveOptions`, `CopyOptions`), data types (`FileMetadata`, `ReadDirectoryEntry`), recursive copy helpers with safety checks
