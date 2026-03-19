# codex-rs/tui/src/onboarding/auth/

This file applies to `codex-rs/tui/src/onboarding/auth/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-tui` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.
- Any user-visible TUI change needs matching snapshot coverage. Mirror behavior in the sibling TUI implementation when the same feature exists there.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-tui`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo insta pending-snapshots -p codex-tui`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Headless ChatGPT login sub-module for the onboarding auth step.

### What this folder does

Contains the headless ChatGPT login implementation that runs the device code OAuth flow without opening a browser automatically. This is used as a fallback when browser-based login is unavailable or when the user needs to complete login on a different device.

### What it plugs into

- **../auth.rs**: The `AuthModeWidget` calls `start_headless_chatgpt_login()` from this module when the user selects the device code login option or when browser-based login fails.
- **codex-login**: Uses `request_device_code()`, `complete_device_code_login()`, `run_login_server()`, and `ServerOptions` for the OAuth device code flow.
- **codex-core**: Uses `AuthManager` to persist authentication credentials after successful login.

### Key files

| File | Role |
|------|------|
| `headless_chatgpt_login.rs` | `start_headless_chatgpt_login()` -- spawns an async task that requests a device code, displays the verification URL and code to the user, and polls for completion. Falls back to a URL-based flow if device code request fails. Renders progress with shimmer animations via `FrameRequester`. |
