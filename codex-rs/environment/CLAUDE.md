# codex-rs/environment/

Filesystem abstraction layer for the Codex executor: trait-based async file operations with a default local implementation backed by Tokio.

## Build & Test
```bash
cargo build -p orbit-code-environment
cargo test -p orbit-code-environment
```

## Architecture

`ExecutorFileSystem` is an async trait defining all file operations the executor can perform (read, write, mkdir, metadata, readdir, remove, copy). `LocalFileSystem` implements this trait using `tokio::fs` for async I/O. The `Environment` struct wraps the filesystem and provides a `get_filesystem()` accessor. This trait boundary enables mock filesystem injection in tests.

## Key Considerations

- Files larger than 512 MB (`MAX_READ_FILE_BYTES`) are rejected by `read_file` -- this is a safety limit to prevent unbounded memory allocation.
- Recursive copy (`copy_dir_recursive`) includes a safety check (`destination_is_same_or_descendant_of_source`) to prevent infinite loops when copying a directory into itself.
- Symlinks are copied as symlinks (not dereferenced) on both Unix and Windows -- platform-specific `copy_symlink` handles the difference.
- The crate has minimal dependencies (only `async-trait`, `orbit-code-utils-absolute-path`, `tokio`) -- keep it lightweight since it sits low in the dependency graph.
