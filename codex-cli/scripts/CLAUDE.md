# codex-cli/scripts/

## Purpose

Build, packaging, and deployment scripts for the `@openai/codex` npm package and its platform-specific variants. These scripts handle downloading native Rust artifacts from CI, bundling them with ripgrep, and producing publishable npm tarballs.

## Key Files

| File | Role |
|------|------|
| `build_npm_package.py` | Stages and packages a single npm module (`codex`, platform variants, or `codex-sdk`). Copies source files, injects version numbers, bundles native binaries from a vendor directory, and optionally runs `npm pack`. |
| `install_native_deps.py` | Downloads native Codex binaries and ripgrep from a GitHub Actions workflow run, extracts archives (zst/tar.gz/zip), and installs them into `vendor/<target>/`. Supports parallel downloads. |
| `build_container.sh` | Builds the Docker image for sandboxed Codex execution: runs `pnpm build`, `pnpm pack`, then `docker build`. |
| `run_in_container.sh` | Launches a Codex Docker container with a mounted work directory, configures allowed domains, initializes the firewall, and runs `codex --full-auto`. |
| `init_firewall.sh` | Configures iptables/ipset inside a Docker container to restrict outbound network access to only allowed domains (defaults to `api.openai.com`). Verifies firewall by testing connectivity. |
| `README.md` | Documents the npm release workflow and how to use `stage_npm_packages.py` from the repo root. |

## How the Build Pipeline Works

1. **`install_native_deps.py`** downloads Rust build artifacts from a GitHub Actions run via `gh run download` and fetches ripgrep binaries from GitHub Releases
2. **`build_npm_package.py`** stages the npm package by copying `bin/codex.js`, the `rg` manifest, `package.json`, and the native binaries from `vendor/` into a staging directory, then produces a tarball
3. **`scripts/stage_npm_packages.py`** (repo root) orchestrates both scripts, expanding meta-packages (e.g., `codex` expands to the base package plus all 6 platform variants)

## Imports From

- `codex-cli/bin/codex.js` and `codex-cli/bin/rg` are copied into staged packages
- `codex-cli/package.json` is read for metadata
- Native binaries come from `codex-rs/` CI build artifacts
- `sdk/typescript/` is used for the SDK package

## Exports To

- Produces npm tarballs written to `dist/npm/` (or specified output directories)
- The Docker image tagged `codex` is built by `build_container.sh`

## Platform Packages Defined

The build system produces these npm packages from a single codex source:

- `@openai/codex` (meta package with optional dependencies)
- `@openai/codex-linux-x64`, `@openai/codex-linux-arm64`
- `@openai/codex-darwin-x64`, `@openai/codex-darwin-arm64`
- `@openai/codex-win32-x64`, `@openai/codex-win32-arm64`
