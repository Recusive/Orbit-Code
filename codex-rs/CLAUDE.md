# codex-rs/

Rust Cargo workspace containing the native implementation of Orbit Code -- TUI, headless execution, app-server, MCP server, sandbox tooling, and all supporting libraries.

## Build & Test

```bash
just codex                    # run from source
just test                     # run tests (cargo-nextest)
just fmt                      # format (rustfmt)
just fix -p <crate>           # clippy fix scoped to one crate
just fix                      # clippy fix workspace-wide (slow)
just write-config-schema      # regenerate config JSON schema
just write-app-server-schema  # regenerate app-server protocol schemas
just bazel-lock-update        # update MODULE.bazel.lock after dep changes
just bazel-lock-check         # verify lockfile is in sync
cargo test -p <crate>         # test a single crate
cargo insta accept -p <crate> # accept snapshot changes
```

## Architecture

The workspace contains 67+ crates organized around a core engine (`orbit-code-core`) that handles AI model interaction, tool execution, and session management. The TUI (`orbit-code-tui`) provides the terminal interface using ratatui. The CLI binary (`orbit-code-cli`) dispatches subcommands (tui, exec, sandbox, mcp-server, app-server). The app-server (`orbit-code-app-server`) exposes a JSON-RPC WebSocket protocol for IDE integrations. The protocol crate (`orbit-code-protocol`) defines shared wire types (Op, EventMsg, Session, Turn).

Key supporting crates handle config loading, secrets management, sandboxing (macOS Seatbelt, Linux landlock), shell command parsing, and OpenTelemetry instrumentation.

## Key Considerations

- **Rust 1.93.0**, edition 2024, pinned in `rust-toolchain.toml`.
- **All crate names** use the `orbit-code-*` prefix (hyphens in package names, underscores in library names).
- **33 clippy lints denied workspace-wide** -- see `[workspace.lints.clippy]` in `Cargo.toml`. No `unwrap()`/`expect()` in library code.
- **Formatter**: `imports_granularity = "Item"` -- one import per `use` statement. Run `just fmt` after every change.
- **Dependencies**: all declared in `[workspace.dependencies]` in root `Cargo.toml`. Per-crate uses `{ workspace = true }`.
- **After dependency changes**: run `just bazel-lock-update` then `just bazel-lock-check`.
- **TUI colors**: ANSI only. `Color::Rgb`, `Color::Indexed`, `.white()`, `.black()`, `.yellow()` are disallowed via clippy.toml.
- **Snapshots**: use `cargo-insta`. Run `cargo insta accept -p <crate>` to accept changes.
- **Mirror rule**: all `tui/` changes must be mirrored in `tui_app_server/` unless documented otherwise.
- **Bazel**: secondary build system for CI/release. If you add `include_str!`, `include_bytes!`, or `sqlx::migrate!`, update the crate's `BUILD.bazel`.
- **Test structure**: integration tests go in `tests/all.rs` -> `tests/suite/mod.rs` -> `tests/suite/*.rs`. Never create multiple top-level test binaries.
