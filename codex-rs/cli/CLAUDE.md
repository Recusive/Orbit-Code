# codex-rs/cli/

The main Codex multitool binary -- the entry point for all CLI subcommands.

## What this folder does

This is the `codex-cli` crate, which produces the `codex` binary. It acts as a dispatcher that routes subcommands to their respective crate implementations: interactive TUI, headless exec, MCP server, app server, sandbox debugging, login/logout, resume/fork, feature management, and more.

## Where it plugs in

- This is the top-level binary crate that ties together nearly all other crates in the workspace
- Distributed via npm (`@openai/codex`), Homebrew, and GitHub Releases
- Uses `codex-arg0` for arg0-based dispatch (platform-specific binary names)

## Imports from

- `codex-tui` / `codex-tui-app-server` -- interactive TUI
- `codex-exec` -- headless non-interactive execution
- `codex-mcp-server` -- MCP server mode
- `codex-app-server` / `codex-app-server-protocol` / `codex-app-server-test-client` -- app server for IDE integrations
- `codex-core` -- config, features, auth, sandbox, terminal info
- `codex-config` -- config file path constants
- `codex-protocol` -- protocol types, config types
- `codex-login` -- login flows
- `codex-execpolicy` -- exec policy checking
- `codex-state` -- state database for memories
- `codex-stdio-to-uds` -- stdio-to-UDS relay
- `codex-rmcp-client` -- MCP client
- `codex-utils-cli` -- CLI config override parsing
- `clap` / `clap_complete` -- argument parsing and shell completions

## Exports to

The `lib.rs` exports `SeatbeltCommand`, `LandlockCommand`, `WindowsCommand`, `debug_sandbox` module, and `login` module -- used by `main.rs`.

## Key files

| File | Role |
|------|------|
| `Cargo.toml` | Crate manifest; binary name `codex`, depends on ~25 workspace crates |
| `src/main.rs` | `MultitoolCli` clap parser with all subcommands; `cli_main` async dispatcher; exit handling, update logic, feature toggles, resume/fork finalization |
| `src/lib.rs` | Exports `SeatbeltCommand`, `LandlockCommand`, `WindowsCommand` clap structs; `debug_sandbox` and `login` modules |
| `src/debug_sandbox.rs` | Sandbox debugging: spawns commands under Seatbelt (macOS), Landlock (Linux), or Windows restricted token |
| `src/debug_sandbox/` | macOS-specific sandbox helpers |
| `src/desktop_app/` | macOS desktop app install/launch logic |
| `src/login.rs` | Login flow implementations |
| `src/mcp_cmd.rs` | MCP subcommand handling |
| `src/exit_status.rs` | Exit status handling helpers |
| `src/wsl_paths.rs` | WSL path normalization (non-Windows) |
| `tests/` | Integration tests for features, MCP, execpolicy, debug clear-memories |
