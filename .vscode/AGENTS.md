# .vscode/

This file applies to `.vscode/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Read the files listed below before changing behavior in this subtree; keep neighboring docs and call sites consistent with any structural change.

## Validate
- Run the nearest package or crate tests that exercise this subtree.

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### What This Folder Does

Provides VS Code workspace settings, recommended extensions, and debug launch configurations for developing the Codex Rust codebase. These files are checked into the repo so all contributors share a consistent editor experience.

### Key Files

| File | Role |
|------|------|
| `settings.json` | Workspace settings for rust-analyzer and TOML formatting. Enables clippy on save, format on save for Rust and TOML files, configures rustfmt with `imports_granularity=Item`, and sets a dedicated `rust-analyzer` target directory to avoid lock contention with Cargo builds. |
| `launch.json` | LLDB debug configurations: (1) "Cargo launch" builds and runs `codex-tui` from `codex-rs/`, (2) "Attach to running codex CLI" attaches to a running process by PID. |
| `extensions.json` | Recommended extensions: `rust-lang.rust-analyzer` (Rust language support), `tamasfe.even-better-toml` (TOML formatting/validation), `vadimcn.vscode-lldb` (LLDB debugger for Rust). |

### What It Plugs Into

- **VS Code**: These files are automatically recognized by VS Code when the repo is opened as a workspace.
- **rust-analyzer**: `settings.json` configures it to run clippy (with `--tests`) on save and use a separate target directory (`codex-rs/target/rust-analyzer`) to avoid blocking `cargo build`.
- **Cargo/Rust**: Debug configs build via `cargo build --bin=codex-tui` in the `codex-rs/` directory.
- **`.devcontainer/`**: The devcontainer config also installs the same extensions (rust-analyzer, even-better-toml) inside the container.

### `settings.json` Details

| Setting | Value | Purpose |
|---------|-------|---------|
| `rust-analyzer.checkOnSave` | `true` | Run checks automatically on file save. |
| `rust-analyzer.check.command` | `clippy` | Use clippy instead of `cargo check` for linting. |
| `rust-analyzer.check.extraArgs` | `["--tests"]` | Include test code in clippy checks. |
| `rust-analyzer.rustfmt.extraArgs` | `["--config", "imports_granularity=Item"]` | Format each import on its own line. |
| `rust-analyzer.cargo.targetDir` | `codex-rs/target/rust-analyzer` | Dedicated output dir to avoid locking conflicts with manual Cargo builds. |
| `evenBetterToml.formatter.reorderArrays` | `false` | Preserves array order in TOML files (important for config files where order matters, e.g., MCP server args, notify settings in `~/.codex/config.toml`). |
| `evenBetterToml.formatter.reorderKeys` | `true` | Alphabetizes TOML keys for consistency. |

### `launch.json` Details

Two LLDB debug configurations:

1. **Cargo launch**: Builds `codex-tui` binary from `codex-rs/` and launches it. Working directory is `${workspaceFolder}/codex-rs`.
2. **Attach to running codex CLI**: Attaches the debugger to an already-running Codex process selected by PID picker. Source language set to Rust.

Both require the `vadimcn.vscode-lldb` extension.

### `extensions.json` Details

Recommended (not required) extensions:
- `rust-lang.rust-analyzer` — Rust IDE support.
- `tamasfe.even-better-toml` — TOML language support and formatting.
- `vadimcn.vscode-lldb` — Native debugger for Rust via LLDB.
- `github.vscode-github-actions` — Commented out; useful for CI workflow editing but not needed by most contributors.
