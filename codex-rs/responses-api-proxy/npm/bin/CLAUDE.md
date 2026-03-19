# codex-rs/responses-api-proxy/npm/bin/

Node.js launcher script for the `codex-responses-api-proxy` native binary.

## What this folder does

Contains the entry point script that detects the host platform and architecture, locates the corresponding pre-compiled Rust binary in the `vendor/` directory, and spawns it with the caller's arguments and stdio inherited.

## Key files

- `codex-responses-api-proxy.js` -- ESM Node.js script:
  - `determineTargetTriple(platform, arch)` -- maps `process.platform` + `process.arch` to Rust target triples: `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`
  - Constructs the binary path: `../vendor/<triple>/codex-responses-api-proxy/codex-responses-api-proxy[.exe]`
  - Spawns the binary via `child_process.spawn()` with `stdio: "inherit"`
  - Forwards `SIGINT`, `SIGTERM`, `SIGHUP` signals to the child process
  - Awaits child exit and mirrors its exit code or signal

## What it plugs into

- Registered as the `codex-responses-api-proxy` bin entry in `package.json`
- Called by the Codex CLI when it needs to start the responses API proxy

## Imports from

- Node.js built-ins: `child_process`, `path`, `url`
- Pre-compiled Rust binary from `../vendor/`

## Exports to

- Acts as the CLI entry point; no programmatic exports
