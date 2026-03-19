# scripts/

This file applies to `scripts/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- This subtree belongs to the `codex-monorepo` package. Keep `package.json` entry points, exports, and scripts aligned with source changes.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex && pnpm format`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### Purpose

Repository-wide utility scripts for CI, release management, code quality checks, and developer tooling. These operate at the monorepo level (as opposed to `codex-cli/scripts/` which is package-specific).

### Key Files

| File | Role |
|------|------|
| `stage_npm_packages.py` | Top-level release orchestrator. Downloads native artifacts from a GitHub Actions workflow, installs them via `codex-cli/scripts/install_native_deps.py`, then invokes `codex-cli/scripts/build_npm_package.py` for each requested package. Writes tarballs to `dist/npm/`. |
| `asciicheck.py` | Linter that checks files for non-ASCII characters. Supports `--fix` mode to replace common Unicode characters (smart quotes, em dashes, non-breaking spaces) with ASCII equivalents. Used in CI to keep source files ASCII-clean. |
| `check_blob_size.py` | CI check that fails if any changed git blob exceeds a configurable size limit (default 500 KiB). Supports an allowlist file. Writes a GitHub Actions step summary. |
| `check-module-bazel-lock.sh` | Verifies that `MODULE.bazel.lock` is up to date by running `bazel mod deps --lockfile_mode=error`. |
| `readme_toc.py` | Checks (and optionally fixes with `--fix`) the Table of Contents in Markdown files. Looks for `<!-- Begin ToC -->` / `<!-- End ToC -->` markers. |
| `mock_responses_websocket_server.py` | Development tool that runs a mock WebSocket server implementing a minimal Responses API endpoint. Used for testing the Codex agent's WebSocket integration locally. |
| `debug-codex.sh` | Developer convenience script that builds and runs the Codex CLI from source via `cargo run --bin codex`. Intended for use as `chatgpt.cliExecutable` in VS Code settings. |
| `install/` | Platform-specific installer scripts (see `scripts/install/CLAUDE.md`) |

### Imports From

- `stage_npm_packages.py` dynamically imports `codex-cli/scripts/build_npm_package.py` to reuse package definitions (`PACKAGE_NATIVE_COMPONENTS`, `PACKAGE_EXPANSIONS`, `CODEX_PLATFORM_PACKAGES`)
- `debug-codex.sh` invokes `cargo run` in the `codex-rs/` directory
- `check_blob_size.py` uses git CLI commands
- `mock_responses_websocket_server.py` depends on the `websockets` Python package

### Exports To

- `stage_npm_packages.py` produces npm tarballs in `dist/npm/`
- CI scripts (`asciicheck.py`, `check_blob_size.py`, `check-module-bazel-lock.sh`, `readme_toc.py`) are invoked by GitHub Actions workflows in `.github/`
