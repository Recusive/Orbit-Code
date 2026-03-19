# codex-rs/cli/src/desktop_app/

This file applies to `codex-rs/cli/src/desktop_app/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-cli` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-cli`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Desktop app installation and launch logic (macOS only).

### What this folder does

Handles finding, downloading, installing, and opening the Codex desktop application. On macOS, it searches standard application directories for `Codex.app`, and if not found, downloads and installs the DMG from a provided URL.

### Where it plugs in

- Called from `app_cmd.rs` in the parent `src/` directory when `codex app` is invoked
- macOS only (guarded by `#[cfg(target_os = "macos")]`)

### Key files

| File | Role |
|------|------|
| `mod.rs` | `run_app_open_or_install` -- public entry point that delegates to platform-specific implementation |
| `mac.rs` | `run_mac_app_open_or_install` -- searches `/Applications/Codex.app` and `~/Applications/Codex.app`; downloads DMG installer if not found; opens the app with `open -a` passing the workspace path |
