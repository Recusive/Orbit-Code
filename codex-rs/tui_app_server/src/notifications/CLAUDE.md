# codex-rs/tui_app_server/src/notifications/

Desktop notification backends for the TUI.

## What this folder does

Provides a `DesktopNotificationBackend` abstraction that sends desktop notifications using terminal escape sequences. Supports two methods: OSC 9 (for terminals like WezTerm and Ghostty) and BEL (universal audible/visual bell). The backend auto-detects the best method based on the terminal environment, or can be explicitly configured.

## What it plugs into

- **../tui.rs**: `Tui` creates and owns the notification backend at startup based on user configuration.
- **../app.rs**: `App` triggers notifications when the agent completes a turn or requires attention.
- **codex_core::config::types::NotificationMethod**: Configuration enum (`Auto`, `Osc9`, `Bel`) that drives backend selection.

## Key files

| File | Role |
|------|------|
| `mod.rs` | `DesktopNotificationBackend` enum and factory; `detect_backend()` function; OSC 9 capability detection via terminal environment variables. |
| `osc9.rs` | `Osc9Backend` -- sends OSC 9 escape sequences for native toast notifications in supported terminals. |
| `bel.rs` | `BelBackend` -- sends the BEL character (`\x07`) for audible/visual bell notifications. |

## Imports from

- `codex_core::config::types::NotificationMethod` -- notification method preference from config.
- Standard library (`std::env`, `std::io`).

## Exports to

- **crate::tui**: `DesktopNotificationBackend`, `detect_backend()`.
