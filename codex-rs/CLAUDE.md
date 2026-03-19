# codex-rs/

Root of the Rust Cargo workspace for the Codex CLI -- a standalone, native executable for AI-assisted coding.

## What this folder does

This is a multi-crate Cargo workspace containing the Rust implementation of the Codex CLI. It provides a terminal UI (TUI), a headless execution mode, an MCP server, sandbox tooling, and all supporting libraries (config, secrets, networking, process hardening, etc.).

## Key crates

| Crate | Path | Purpose |
|-------|------|---------|
| `codex-core` | `core/` | Business logic engine; library for building Codex-powered apps |
| `codex-tui` | `tui/` | Fullscreen Ratatui-based terminal UI |
| `codex-cli` | `cli/` | Multitool binary that dispatches subcommands (tui, exec, sandbox, mcp-server, etc.) |
| `codex-exec` | `exec/` | Headless/non-interactive CLI for automation |
| `codex-app-server` | `app-server/` | App server for IDE integrations (MCP-based protocol) |
| `codex-protocol` | `protocol/` | Core protocol types (Op, EventMsg, Session, Turn) |
| `codex-config` | `config/` | Configuration loading and layered merge from TOML files |
| `codex-secrets` | `secrets/` | Encrypted secrets management with keyring backend |

## Workspace configuration

- **Rust edition**: 2024 (workspace-wide default)
- **Toolchain**: Rust 1.93.0 (pinned in `rust-toolchain.toml`)
- **Formatter**: `rustfmt.toml` -- edition 2024, `imports_granularity = "Item"`
- **Linter**: Extensive Clippy deny rules in `Cargo.toml` `[workspace.lints.clippy]`; `clippy.toml` adds disallowed ratatui color methods and a 256-byte `large-error-threshold`
- **License auditing**: `deny.toml` for `cargo-deny` (advisories, licenses, bans, sources)
- **Release profile**: LTO fat, strip symbols, single codegen unit

## Build system

- **Primary**: Cargo (`cargo build`, `cargo test`)
- **Secondary**: Bazel (experimental; see `docs/bazel.md` and `BUILD.bazel` files)
- **Test runner**: `cargo-nextest` (configured in `.config/nextest.toml`)

## Imports / exports

- The workspace depends on ~150 external crates (see `[workspace.dependencies]` in `Cargo.toml`)
- Several crates are patched via `[patch.crates-io]`: crossterm, ratatui, tokio-tungstenite, tungstenite (all from forks)
- The `cli/` binary is the main deliverable; it is distributed via npm (`@openai/codex`), Homebrew, and GitHub Releases

## Key files

- `Cargo.toml` -- workspace definition, all member crates, shared dependencies, lint config, release profile
- `Cargo.lock` -- locked dependency versions
- `rust-toolchain.toml` -- pinned Rust toolchain
- `clippy.toml` -- custom Clippy configuration
- `rustfmt.toml` -- formatter settings
- `deny.toml` -- cargo-deny configuration for license/advisory/ban/source checks
- `README.md` -- user-facing documentation for installing and using the CLI
- `config.md` -- detailed configuration reference
