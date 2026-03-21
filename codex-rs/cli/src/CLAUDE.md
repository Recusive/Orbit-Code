# codex-rs/cli/src/

Source for the `orbit-code` binary and `orbit_code_cli` library.

`main.rs` defines `MultitoolCli` (clap) with ~18 subcommands and the `cli_main` async dispatcher. `lib.rs` exports sandbox command structs and the `login`/`debug_sandbox` modules. Platform-gated: `app_cmd.rs` and `desktop_app/` are macOS-only, `wsl_paths.rs` is non-Windows.
