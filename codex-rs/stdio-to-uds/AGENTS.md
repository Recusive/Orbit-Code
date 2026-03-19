# codex-rs/stdio-to-uds/

This file applies to `codex-rs/stdio-to-uds/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-stdio-to-uds` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-stdio-to-uds`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Adapter that bridges stdio to Unix Domain Sockets.

### What this folder does

Provides a library and CLI binary (`codex-stdio-to-uds`) that connects to a Unix Domain Socket and relays data bidirectionally between stdin/stdout and the socket. This enables using UDS-based MCP servers with Codex's stdio transport.

### What it plugs into

- Users configure Codex to use a UDS-based MCP server by specifying `codex-stdio-to-uds` as the command with the socket path as argument: `codex --config mcp_servers.example={command="codex-stdio-to-uds",args=["/tmp/mcp.sock"]}`.
- Works cross-platform: uses `std::os::unix::net::UnixStream` on Unix and `uds_windows` crate on Windows.

### Imports from

- `anyhow` -- error handling.
- `uds_windows` (Windows only) -- Unix domain socket support on Windows.

### Exports to

- `codex_stdio_to_uds::run(socket_path)` -- the library function that performs the relay.
- `codex-stdio-to-uds` binary -- CLI wrapper.

### Key files

- `Cargo.toml` -- crate manifest with platform-specific dependencies.
- `README.md` -- documentation explaining the UDS transport adapter.
- `src/lib.rs` -- `run()` function: connects to the socket, spawns a thread to copy socket-to-stdout, copies stdin-to-socket in the main thread, then half-closes and joins.
- `src/main.rs` -- CLI binary: parses the single socket-path argument and calls `run()`.
- `tests/stdio_to_uds.rs` -- integration test that spawns a UDS server, runs the binary, and verifies bidirectional data transfer.
