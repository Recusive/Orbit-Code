# codex-rs/windows-sandbox-rs/

This file applies to `codex-rs/windows-sandbox-rs/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-windows-sandbox` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-windows-sandbox`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Windows sandbox implementation for Codex command execution.

### What this folder does

Provides the Windows-specific sandbox that runs AI-proposed commands under a restricted security token. Implements ACL manipulation, restricted token creation, ConPTY process spawning, firewall-based network blocking, workspace write protection, DPAPI encryption, and an elevated runner for higher-privilege sandbox operations. On non-Windows platforms, all public functions return stub errors.

### What it plugs into

- Used by `codex-core` as the Windows sandbox backend for command execution.
- The `codex-windows-sandbox-setup` binary performs one-time elevated setup (user creation, ACL configuration).
- The `codex-command-runner` binary runs as the sandbox user in the elevated path.

### Imports from

- `codex-protocol` -- `SandboxPolicy`.
- `codex-utils-pty` -- `RawConPty` for ConPTY creation.
- `codex-utils-absolute-path` -- path normalization.
- `codex-utils-string` -- string utilities.
- `windows-sys` -- Win32 API bindings (threading, security, pipes, console, etc.).
- `windows` -- COM/WMI for firewall management.
- `serde`, `serde_json`, `base64`, `chrono`, `rand`, `tempfile`, `dunce`.

### Exports to

- `run_windows_sandbox_capture()` -- runs a command in the sandbox and captures output.
- `run_windows_sandbox_legacy_preflight()` -- pre-configures ACLs for workspace-write mode.
- `SandboxPolicy`, `parse_policy()` -- policy parsing.
- ACL helpers: `add_deny_write_ace()`, `ensure_allow_mask_aces()`, `fetch_dacl_handle()`, etc.
- Token helpers: `create_readonly_token_with_cap_from()`, `create_workspace_write_token_with_caps_from()`, etc.
- `spawn_conpty_process_as_user()` -- ConPTY process spawning.
- `ipc_framed` module -- framed IPC protocol for the elevated runner.
- Setup: `run_elevated_setup()`, `run_setup_refresh()`, `sandbox_setup_is_complete()`.

### Key files

- `Cargo.toml` -- crate manifest with extensive `windows-sys` feature flags.
- `build.rs` -- Windows resource manifest embedding (winres).
- `src/lib.rs` -- module declarations, Windows/stub conditional compilation, and `run_windows_sandbox_capture()` implementation.
- `src/bin/setup_main.rs` -- `codex-windows-sandbox-setup` binary entrypoint.
- `src/bin/command_runner.rs` -- `codex-command-runner` binary entrypoint.
