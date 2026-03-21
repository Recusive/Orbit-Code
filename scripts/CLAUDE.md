# scripts/

Repository-wide utility scripts for CI, release management, and developer tooling. These operate at the monorepo level (as opposed to `codex-cli/scripts/` which is package-specific).

## Architecture

The release pipeline centers on `stage_npm_packages.py`, which downloads native artifacts from GitHub Actions and invokes `codex-cli/scripts/build_npm_package.py` to produce npm tarballs in `dist/npm/`. CI quality scripts (`asciicheck.py`, `check_blob_size.py`, `readme_toc.py`, `check-module-bazel-lock.sh`) are invoked by GitHub Actions workflows.

## Module Layout

- **Release**: `stage_npm_packages.py` (top-level orchestrator for npm tarball production)
- **CI linters**: `asciicheck.py` (non-ASCII detection), `check_blob_size.py` (blob size limits), `readme_toc.py` (ToC validation), `check-module-bazel-lock.sh` (Bazel lockfile sync)
- **Dev tools**: `debug-codex.sh` (build and run CLI from source), `mock_responses_websocket_server.py` (mock WebSocket server for local testing)
- **Installers**: `install/` (platform-specific standalone installers that bypass npm)

## Key Considerations

- `stage_npm_packages.py` dynamically imports from `codex-cli/scripts/build_npm_package.py` -- the two must stay in sync.
- `mock_responses_websocket_server.py` requires the `websockets` Python package.
- `asciicheck.py` supports `--fix` mode for common Unicode-to-ASCII replacements.
