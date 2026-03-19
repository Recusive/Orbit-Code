# codex-cli/bin/

This file applies to `codex-cli/bin/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- This subtree belongs to the `@openai/codex` package. Keep `package.json` entry points, exports, and scripts aligned with source changes.

## Validate
- Run the nearest package or crate tests that exercise this subtree.

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### Purpose

Contains the executable entry points shipped inside the `@openai/codex` npm package.

### Key Files

| File | Role |
|------|------|
| `codex.js` | Main CLI entry point (ESM). Detects OS/arch, resolves the native `codex` binary from `vendor/`, and spawns it with full signal forwarding (SIGINT, SIGTERM, SIGHUP). Mirrors the child process exit code/signal in the parent. |
| `rg` | DotSlash manifest (JSON) describing how to fetch ripgrep 15.1.0 binaries for each supported platform. Used by the build system to bundle `rg` alongside the CLI. |

### How `codex.js` Works

1. Maps `process.platform` + `process.arch` to a Rust target triple (e.g., `aarch64-apple-darwin`)
2. Attempts to `require.resolve()` the platform-specific optional dependency package (e.g., `@openai/codex-darwin-arm64`)
3. Falls back to a local `vendor/` directory if the optional dependency is not installed
4. Prepends the `vendor/<target>/path/` directory to `$PATH` (provides the bundled `rg`)
5. Spawns the native binary asynchronously with `stdio: "inherit"` and forwards signals

### Imports From

- Node.js built-ins only: `child_process`, `fs`, `module`, `path`, `url`

### Exports To

- This directory is referenced by `codex-cli/package.json` as `"bin": { "codex": "bin/codex.js" }`
- The `rg` manifest is consumed by `codex-cli/scripts/install_native_deps.py` during the build process
