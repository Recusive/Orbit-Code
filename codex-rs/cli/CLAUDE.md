# codex-rs/cli/

Main binary entry point for Orbit Code. Dispatches subcommands to their respective crate implementations.

## Build & Test

```bash
cargo build -p orbit-code          # Build the binary
cargo test -p orbit-code           # Run tests (integration tests in tests/)
just fmt                           # Format after changes
just fix -p orbit-code             # Clippy
```

## Architecture

The `MultitoolCli` struct in `main.rs` defines all subcommands via clap. The `cli_main` function dispatches to the appropriate crate:

- No subcommand (default) -> interactive TUI (`orbit-code-tui`)
- `exec` / `e` -> headless execution (`orbit-code-exec`)
- `review` -> code review via exec
- `mcp-server` -> MCP server mode (`orbit-code-mcp-server`)
- `app-server` -> IDE integration server (`orbit-code-app-server`)
- `sandbox` -> run commands under platform sandbox
- `login` / `logout` -> auth credential management (`orbit-code-login`)
- `resume` / `fork` -> continue or branch previous sessions
- `mcp` -> manage external MCP servers
- `features` -> inspect feature flags
- `completion` -> shell completions
- `debug` -> debugging tools (app-server, clear-memories)

The `lib.rs` exports sandbox command structs (`SeatbeltCommand`, `LandlockCommand`, `WindowsCommand`) and the `login` and `debug_sandbox` modules. These are consumed by `main.rs`.

Platform-specific modules: `app_cmd.rs` and `desktop_app/` are macOS-only. `wsl_paths.rs` is non-Windows only.

## Key Considerations

- Package name is `orbit-code`, NOT `orbit-code-cli`. Library name is `orbit_code_cli`. The binary is also named `orbit-code`.
- This crate depends on ~25 workspace crates. It is a thin dispatcher -- business logic lives in the downstream crates.
- `main.rs` handles arg0-based dispatch via `orbit-code-arg0` (platform-specific binary names like `orbit-code-linux-sandbox` route to sandbox logic instead of normal CLI flow).
- Resume/fork finalization and update checks happen after the TUI exits in `cli_main`.
- Integration tests live in `tests/` and cover features, MCP, execpolicy, and debug commands.
