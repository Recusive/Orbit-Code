# codex-rs/.github/workflows/

This file applies to `codex-rs/.github/workflows/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.

## Validate
- Run the nearest package or crate tests that exercise this subtree.

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

GitHub Actions workflow definitions for the codex-rs Rust workspace.

### What this folder does

Contains CI workflow YAML files that GitHub Actions executes on pull requests and pushes to `main`.

### Key files

- `cargo-audit.yml` -- Runs `cargo audit --deny warnings` to check for known security advisories in dependencies.
  - **Trigger**: Pull requests and pushes to `main`
  - **Runner**: `ubuntu-latest`
  - **Working directory**: `codex-rs`
  - **Steps**: Checkout, install stable Rust toolchain, install `cargo-audit` via `taiki-e/install-action`, run audit
  - **Permissions**: Read-only contents access

### What it plugs into

- GitHub Actions reads these files to create CI check runs
- The audit workflow respects suppressions defined in `codex-rs/.cargo/audit.toml` and `codex-rs/deny.toml`

### Imports from / exports to

- No code imports; these are CI pipeline definitions
- The audit results appear as GitHub check statuses on PRs
