# codex-rs/tui_app_server/src/onboarding/

First-run onboarding flow: welcome screen, authentication, and directory trust.

## What this folder does

Implements the multi-step onboarding experience shown when a user first runs the CLI or enters an untrusted directory. The flow includes a welcome animation screen, OpenAI authentication (device code or API key), and a directory trust decision prompt. Each step is a self-contained widget driven by a shared `OnboardingScreen` state machine.

## What it plugs into

- **../lib.rs**: `run_ratatui_app()` calls `run_onboarding_app()` before entering the main event loop when onboarding is needed.
- **../app_server_session.rs**: Authentication steps communicate with the app server for login flows.
- **codex_core::config**: Trust decisions are persisted via `set_project_trust_level()`.
- **codex_login**: Device code login flow types.

## Key files

| File | Role |
|------|------|
| `mod.rs` | Module root; re-exports `TrustDirectorySelection` and declares submodules. |
| `onboarding_screen.rs` | `OnboardingScreen` state machine -- manages the step sequence (Welcome -> Auth -> TrustDirectory), keyboard routing, rendering, and result collection. |
| `welcome.rs` | `WelcomeWidget` -- animated ASCII art welcome screen with "press Enter to continue" prompt. |
| `auth.rs` | `AuthModeWidget` -- sign-in screen supporting device code flow, API key entry, and ChatGPT browser login. Communicates with the app server for login RPCs. |
| `auth/` | Subdirectory with headless ChatGPT login helper. |
| `trust_directory.rs` | `TrustDirectoryWidget` -- prompts the user to trust or distrust the current working directory, persisting the decision to config. |

## Imports from

- `crate::LoginStatus` -- current authentication state.
- `crate::app_server_session::AppServerSession` -- for login RPCs.
- `crate::tui` -- `Tui`, `FrameRequester`, `TuiEvent`.
- `crate::ascii_animation` -- for the welcome screen animation.
- `crate::selection_list` -- for trust decision option rows.
- `crate::render` -- `Insets`, `Renderable`, rendering utilities.
- `codex_app_server_client` -- `AppServerRequestHandle`.
- `codex_app_server_protocol` -- login and account notification types.
- `codex_core::config` -- trust level persistence.
- `codex_core::git_info` -- git project root resolution for trust scope.
- `codex_login::DeviceCode` -- device code login types.

## Exports to

- **crate::lib** / **crate::app**: `run_onboarding_app()`, `OnboardingScreenArgs`, `TrustDirectorySelection`.
