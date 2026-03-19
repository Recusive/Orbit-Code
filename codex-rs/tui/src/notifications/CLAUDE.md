# codex-rs/tui/src/notifications/

Desktop notification backends for the TUI.

## What this folder does

Provides platform-aware desktop notification support through two backends: OSC 9 (for terminals that support it like iTerm2, WezTerm, Ghostty, Kitty) and BEL (the terminal bell, as a universal fallback). The module auto-detects which backend to use based on the terminal environment, or respects an explicit user configuration.

## What it plugs into

- **../tui.rs**: The `Tui` struct creates a `DesktopNotificationBackend` during initialization and uses it to send notifications when the agent completes work or needs attention.
- **codex-core**: Uses `NotificationMethod` from config types to determine the preferred notification method.

## Key files

| File | Role |
|------|------|
| `mod.rs` | `DesktopNotificationBackend` enum and `detect_backend()` factory. Contains `supports_osc9()` which checks `TERM_PROGRAM`, `ITERM_SESSION_ID`, `TERM`, and `WT_SESSION` environment variables to determine terminal support. Also includes tests for detection logic. |
| `osc9.rs` | `Osc9Backend` -- sends OSC 9 escape sequences (`ESC ] 9 ; message ST`) for rich desktop notifications with message text. |
| `bel.rs` | `BelBackend` -- sends the BEL character (`\x07`) as a simple audible/visual alert. Message content is ignored. |

## Detection heuristic

1. If `WT_SESSION` is set (Windows Terminal), fall back to BEL (OSC 9 not supported).
2. If `TERM_PROGRAM` is `WezTerm` or `ghostty`, use OSC 9.
3. If `ITERM_SESSION_ID` is set, use OSC 9.
4. If `TERM` is `xterm-kitty`, `wezterm`, or `wezterm-mux`, use OSC 9.
5. Otherwise, fall back to BEL.
