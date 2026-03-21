# scripts/install/

Platform-specific standalone installers that download the Codex CLI binary from GitHub Releases without requiring npm or Node.js.

## Architecture

Both scripts follow the same pattern: detect OS/architecture, query the GitHub API for the latest release version (or use a provided version), download the platform-specific npm tarball from GitHub Releases, extract the native binaries, install them to a local directory, and update `$PATH`.

## Module Layout

- **install.sh** -- POSIX shell installer for macOS and Linux. Installs to `~/.local/bin` (or `$CODEX_INSTALL_DIR`). Handles Rosetta detection on macOS.
- **install.ps1** -- PowerShell installer for Windows. Installs to `%LOCALAPPDATA%\Programs\OpenAI\Codex\bin` (or `$env:CODEX_INSTALL_DIR`).

## Key Considerations

- The tarball structure these scripts expect must match what `codex-cli/scripts/build_npm_package.py` produces.
- Both scripts accept an optional version argument (defaults to `latest`).
- The GitHub API endpoint used is `api.github.com/repos/openai/codex/releases/latest`.
