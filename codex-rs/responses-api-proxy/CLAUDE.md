# codex-rs/responses-api-proxy/

Minimal HTTP proxy (`codex-responses-api-proxy`) that forwards requests to the OpenAI Responses API with injected authentication.

## What this folder does

Provides a lightweight, single-purpose HTTP proxy that:
- Reads an API key securely from stdin (using low-level `read(2)` on Unix to avoid buffered copies in memory)
- Locks the key in memory with `mlock(2)` to prevent swapping
- Binds to a local TCP port (configurable or ephemeral)
- Forwards `POST /v1/responses` requests to a configurable upstream URL (defaults to `https://api.openai.com/v1/responses`)
- Injects the `Authorization: Bearer <key>` header, stripping any client-supplied auth
- Streams the upstream response back to the client
- Rejects all other HTTP methods and paths with 403
- Optionally writes a `server_info.json` with port and PID for parent process coordination
- Optionally supports `GET /shutdown` for graceful termination

This proxy is used by the Node.js CLI to keep the API key out of the main process's environment and argument list.

## What it plugs into

- Spawned by the Codex CLI (Node.js layer) as a child process
- The CLI pipes the API key via stdin and reads the server info file to discover the proxy's port
- All Responses API traffic from the CLI is routed through this proxy

## Imports from

- `codex-process-hardening` -- `pre_main_hardening()` for process security
- `clap` -- CLI argument parsing
- `reqwest` -- blocking HTTP client for upstream forwarding
- `tiny_http` -- lightweight HTTP server for local listening
- `zeroize` -- secure memory zeroing for the API key buffer
- `serde`, `serde_json` -- server info serialization
- `anyhow` -- error handling

## Exports to

- Binary: `codex-responses-api-proxy` (compiled Rust executable)
- Library: `codex_responses_api_proxy::run_main(args)` for programmatic invocation
- npm package: `@openai/codex-responses-api-proxy` wraps the binary for Node.js distribution

## Key files

- `Cargo.toml` -- crate definition with both library and binary targets
- `src/lib.rs` -- `Args` struct (clap), `run_main()` entry point, request forwarding logic, server info writing
- `src/main.rs` -- binary entry point; calls `pre_main_hardening()` then `run_main()`
- `src/read_api_key.rs` -- secure stdin API key reading with `mlock(2)`, validation, and buffer zeroization
- `npm/` -- npm package for distribution
