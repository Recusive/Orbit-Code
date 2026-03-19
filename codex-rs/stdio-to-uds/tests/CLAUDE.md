# codex-rs/stdio-to-uds/tests/

Integration tests for the stdio-to-UDS adapter.

## What this folder does

Tests the `codex-stdio-to-uds` binary end-to-end by creating a temporary Unix socket, spawning the binary, and verifying bidirectional data transfer.

## Key files

- `stdio_to_uds.rs` -- `pipes_stdin_and_stdout_through_socket`: creates a UDS listener in a temp directory, spawns the `codex-stdio-to-uds` binary with the socket path argument, sends "request" via stdin, verifies the server receives it, sends "response" from the server, and verifies it appears on stdout. Includes timeout handling and diagnostic event collection for flaky test debugging.

## Imports from

- `codex-utils-cargo-bin` -- locates the compiled binary.
- `tempfile` -- creates temporary directories for socket files.
- `anyhow`, `pretty_assertions`.
