# scripts/install/

This file applies to `scripts/install/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- This subtree belongs to the `codex-monorepo` package. Keep `package.json` entry points, exports, and scripts aligned with source changes.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex && pnpm format`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### Purpose

Platform-specific installer scripts that download and install the Codex CLI binary directly from GitHub Releases, without requiring npm or Node.js. These provide an alternative installation path for users who prefer a standalone binary.

### Key Files

| File | Role |
|------|------|
| `install.sh` | POSIX shell installer for macOS and Linux. Detects OS/architecture, resolves the latest release version from the GitHub API, downloads the platform-specific npm tarball, extracts the `codex` and `rg` binaries to `~/.local/bin` (or `$CODEX_INSTALL_DIR`), and configures `$PATH` in the user's shell profile. |
| `install.ps1` | PowerShell installer for Windows. Detects architecture (x64/ARM64), downloads the platform tarball, extracts `codex.exe`, `rg.exe`, `codex-command-runner.exe`, and `codex-windows-sandbox-setup.exe` to `%LOCALAPPDATA%\Programs\OpenAI\Codex\bin` (or `$env:CODEX_INSTALL_DIR`), and updates the user's `PATH` environment variable. |

### How They Work

1. Accept an optional version argument (defaults to `latest`)
2. Query the GitHub API (`api.github.com/repos/openai/codex/releases/latest`) to resolve the version tag
3. Download the platform-specific npm tarball from GitHub Releases (e.g., `codex-npm-darwin-arm64-0.6.0.tgz`)
4. Extract native binaries from the `package/vendor/<target>/` path inside the tarball
5. Install to the target directory and update `$PATH`

### Supported Platforms

- **install.sh**: macOS (Intel + Apple Silicon, with Rosetta detection), Linux (x64 + ARM64)
- **install.ps1**: Windows (x64 + ARM64)

### Relationship to Other Directories

- Downloads artifacts produced by `codex-cli/scripts/build_npm_package.py` and published to GitHub Releases
- The tarball structure matches what `codex-cli/scripts/build_npm_package.py` produces for platform packages
