# Codex CLI — Monorepo Root

## What This Is
Codex CLI is OpenAI's local coding agent CLI tool (similar to Claude Code). This is the monorepo root containing all packages, crates, and tooling.

## Repository Structure

| Directory | Purpose |
|-----------|---------|
| `codex-cli/` | Legacy TypeScript CLI (npm package `@openai/codex`) |
| `codex-rs/` | **Primary codebase** — Rust implementation of the CLI, TUI, core engine, and all subsystems |
| `sdk/` | TypeScript SDK for programmatic access to the Codex agent |
| `shell-tool-mcp/` | MCP server that exposes shell tool capabilities |
| `scripts/` | Repo-wide utility scripts (release, CI helpers) |
| `docs/` | Documentation (contributing, install, open-source fund) |
| `tools/` | Developer tooling (argument-comment linting) |
| `patches/` | pnpm patch overrides for dependencies |
| `third_party/` | Vendored third-party code |
| `lancedb/` | LanceDB related files |
| `.codex/` | Codex skills (babysit-pr, test-tui) |
| `.github/` | GitHub Actions workflows, issue templates, CI scripts |

## Key Files at Root

| File | Purpose |
|------|---------|
| `package.json` | pnpm workspace root — formatting scripts, engine requirements (Node ≥22) |
| `pnpm-workspace.yaml` | Defines workspace packages: `codex-cli`, `sdk/typescript`, `shell-tool-mcp`, `codex-rs/responses-api-proxy/npm` |
| `justfile` | Task runner for Rust development — build, test, format, lint, schema generation |
| `MODULE.bazel` | Bazel build configuration for remote builds and CI |
| `flake.nix` | Nix development environment |
| `AGENTS.md` | Coding conventions for Rust code, TUI styling, test patterns, app-server API rules |
| `defs.bzl` | Bazel macro definitions |
| `cliff.toml` | Changelog generation config (git-cliff) |
| `announcement_tip.toml` | CLI announcement/tips configuration |

## Build Systems
- **Rust**: Cargo (primary local dev) + Bazel (CI/release builds)
- **TypeScript**: pnpm + esbuild/Vite
- **Task runner**: `just` (justfile in root, working dir set to `codex-rs/`)

## Common Commands
```bash
just codex          # Run codex CLI from source
just test           # Run Rust tests (nextest)
just fmt            # Format Rust code
just fix            # Run clippy fixes
just write-config-schema    # Regenerate config JSON schema
just write-app-server-schema # Regenerate app-server protocol schemas
```

## Architecture Overview
The Codex CLI is primarily a **Rust application** (`codex-rs/`) that:
1. Provides a terminal UI (TUI) built with `ratatui`
2. Connects to OpenAI's API via a backend client
3. Executes tools (shell commands, file operations) in sandboxed environments
4. Manages sessions, conversations, and agent state
5. Exposes an app-server for IDE integrations

The legacy TypeScript CLI (`codex-cli/`) is being superseded by the Rust implementation.
