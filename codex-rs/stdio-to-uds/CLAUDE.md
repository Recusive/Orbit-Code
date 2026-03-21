# codex-rs/stdio-to-uds/

Adapter (library + CLI binary) that bridges stdin/stdout to a Unix Domain Socket bidirectionally. Enables using UDS-based MCP servers with Codex's stdio transport.

## Build & Test
```bash
cargo build -p orbit-code-stdio-to-uds
cargo test -p orbit-code-stdio-to-uds
```

## Key Considerations
- Cross-platform: uses `std::os::unix::net::UnixStream` on Unix, `uds_windows` crate on Windows.
