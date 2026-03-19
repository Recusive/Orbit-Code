# codex-rs/tui_app_server/src/onboarding/auth/

Headless ChatGPT authentication helper for the onboarding flow.

## What this folder does

Contains the headless ChatGPT login implementation used during the onboarding authentication step. This handles browser-based ChatGPT authentication in environments where a full browser UI is not available.

## What it plugs into

- **../auth.rs**: The `AuthModeWidget` delegates to this module for ChatGPT browser-based login when that sign-in method is selected.
- **codex_app_server_protocol**: Login and account API types.

## Key files

| File | Role |
|------|------|
| `headless_chatgpt_login.rs` | Implements headless ChatGPT login -- opens a browser for OAuth, polls for completion, and returns auth tokens to the onboarding flow. |

## Imports from

- `codex_app_server_protocol` -- login types.
- Standard library and async runtime.

## Exports to

- **../auth.rs**: Login result types used by the authentication widget.
