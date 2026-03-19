# codex-rs/.github/

This file applies to `codex-rs/.github/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.

## Validate
- Run the nearest package or crate tests that exercise this subtree.

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

GitHub-specific configuration for the codex-rs workspace.

### What this folder does

Contains GitHub Actions workflow definitions for CI/CD automation specific to the Rust workspace.

### Structure

- `workflows/` -- GitHub Actions workflow files

### What it plugs into

- GitHub Actions reads workflow files from this directory to run CI checks on pull requests and pushes to `main`
- Note: The top-level repository `.github/` may also contain workflows; this nested `.github/` specifically targets the `codex-rs/` workspace

### Key files

- `workflows/cargo-audit.yml` -- The sole remaining workflow; runs `cargo audit` on PRs and main pushes
