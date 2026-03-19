# codex-rs/responses-api-proxy/src/

This file applies to `codex-rs/responses-api-proxy/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-responses-api-proxy` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-responses-api-proxy`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source code for the `codex-responses-api-proxy` crate.

### What this folder does

Contains the Rust implementation of the minimal HTTP proxy for the OpenAI Responses API.

### Key files

- `lib.rs` -- main library entry point:
  - `Args` struct (clap-derived): `--port`, `--server-info`, `--http-shutdown`, `--upstream-url`
  - `run_main(args)` -- reads API key from stdin, binds TCP listener, starts `tiny_http` server, spawns handler threads
  - `forward_request()` -- validates only `POST /v1/responses` is allowed (rejects all else with 403), reads request body, builds upstream headers (strips client Auth/Host, injects server-side Bearer token marked as sensitive), forwards via `reqwest::blocking::Client`, streams response back
  - `bind_listener()` -- binds to `127.0.0.1` on specified or ephemeral port
  - `write_server_info()` -- writes `{"port": N, "pid": N}` JSON file for parent process discovery
- `main.rs` -- binary entry point: calls `codex_process_hardening::pre_main_hardening()` via `#[ctor]`, then parses args and calls `run_main()`
- `read_api_key.rs` -- secure API key ingestion:
  - `read_auth_header_from_stdin()` -- reads key from stdin into a stack buffer, prepends "Bearer ", validates (ASCII alphanumeric + `-`/`_` only), creates a leaked `&'static str`, and locks it in memory with `mlock(2)`
  - Uses raw `libc::read()` on Unix to avoid Rust's buffered `stdin()` (prevents extra in-memory copies of the key)
  - `mlock_str()` -- page-aligned `mlock(2)` to prevent the key from being swapped to disk
  - `validate_auth_header_bytes()` -- ensures key contains only safe ASCII characters
  - Comprehensive test suite covering normal reads, short reads, newline trimming, empty input, buffer overflow, IO errors, invalid UTF-8, and invalid characters

### Imports from

- `codex-process-hardening` -- pre-main security hardening
- `clap` -- argument parsing
- `reqwest` -- HTTP client (blocking mode, no timeout for streaming)
- `tiny_http` -- lightweight HTTP server
- `zeroize` -- secure memory zeroing
- `serde`, `serde_json` -- JSON serialization
- `anyhow` -- error handling
- `libc` -- raw stdin read and mlock

### Exports to

- `run_main(Args)` -- library entry point used by the binary target
- `Args` -- CLI argument struct (public for programmatic use)
