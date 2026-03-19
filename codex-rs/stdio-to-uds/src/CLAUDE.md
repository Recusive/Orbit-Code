# codex-rs/stdio-to-uds/src/

Source code for the `codex-stdio-to-uds` crate.

## What this folder does

Contains the library implementation and binary entrypoint for the stdio-to-UDS adapter.

## Key files

- `lib.rs` -- `run(socket_path)`: connects to a Unix domain socket, spawns a thread to copy socket data to stdout, copies stdin to the socket in the calling thread, performs a half-close on the write side, and waits for the stdout thread to finish.
- `main.rs` -- CLI binary that validates a single command-line argument (socket path) and calls `run()`.

## Imports from

- `std::os::unix::net::UnixStream` (Unix) or `uds_windows::UnixStream` (Windows).
- `anyhow` for error handling.
