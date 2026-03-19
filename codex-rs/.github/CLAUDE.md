# codex-rs/.github/

GitHub-specific configuration for the codex-rs workspace.

## What this folder does

Contains GitHub Actions workflow definitions for CI/CD automation specific to the Rust workspace.

## Structure

- `workflows/` -- GitHub Actions workflow files

## What it plugs into

- GitHub Actions reads workflow files from this directory to run CI checks on pull requests and pushes to `main`
- Note: The top-level repository `.github/` may also contain workflows; this nested `.github/` specifically targets the `codex-rs/` workspace

## Key files

- `workflows/cargo-audit.yml` -- The sole remaining workflow; runs `cargo audit` on PRs and main pushes
