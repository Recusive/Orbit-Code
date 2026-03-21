# Multi-Provider Auth Switching

> **Date:** 2026-03-21
> **Status:** Approved
> **Approach:** TUI-Only (Approach A)

## Problem

Users cannot switch between auth methods (API key vs OAuth) or add credentials for a new provider mid-session. If a user starts with OpenAI OAuth and wants to switch to a Claude model, there is no UI to enter Anthropic credentials. The backend storage (V2 auth) already supports multiple providers side-by-side, but the TUI has no flow to exercise it.

## Design Decisions

| Decision | Choice |
|----------|--------|
| When auth picker appears | Always when switching providers, even if credentials exist |
| Auth methods per provider | Full matrix: API key + OAuth for both OpenAI and Anthropic |
| Popup structure | Three-step sequential: model -> effort -> auth method |
| API key entry | Inline masked text input in TUI popup |
| OAuth entry | Sub-menu: Browser Login or Paste Token inline |
| Auth persistence | Persist preferred auth mode per-provider, pre-highlight on next switch |
| Standalone management | Dedicated `/auth` command for full credential management |

## Data Model

### Existing (no changes)

```
AuthDotJsonV2 {
  providers: HashMap<ProviderName, ProviderAuth>
}

ProviderName: OpenAI | Anthropic
ProviderAuth: OpenAiApiKey | Chatgpt | ChatgptAuthTokens | AnthropicApiKey | AnthropicOAuth
```

### New Addition

```
AuthDotJsonV2 {
  providers: HashMap<ProviderName, ProviderAuth>,
  preferred_auth_modes: HashMap<ProviderName, AuthMode>  // NEW
}
```

`preferred_auth_modes` records the last-used auth method per provider. Read/written alongside credentials. Merge-on-save preserves existing preferences (same pattern as provider merge).

## Model Switch Flow

### Current (two steps)

```
/model -> pick model -> pick effort -> apply
```

### New (three steps)

```
/model -> pick model -> pick effort -> auth method popup -> apply
```

### Auth Step Logic

```
User picks effort -> determine target provider
  |-- Same provider as current? -> skip auth popup, apply immediately
  |-- Different provider?
       -> Build auth popup with context-aware items
       -> User picks method
       -> If new credentials needed, show inline input
       -> Save credentials + preference
       -> Apply model switch
```

## Auth Popup UI

### No credentials exist for target provider

```
Select Authentication for Anthropic

> 1. API Key          Enter your Anthropic API key
  2. OAuth Login      Sign in with your Anthropic account

  Press enter to confirm or esc to go back.
```

### API key already stored

```
Select Authentication for Anthropic

> 1. API Key (current)   sk-ant-*******dk3F
  2. Enter new API Key   Replace with a different key
  3. OAuth Login         Sign in with your Anthropic account

  Press enter to confirm or esc to go back.
```

### OAuth already stored

```
Select Authentication for OpenAI

  1. API Key             Enter your OpenAI API key
> 2. OAuth (current)     Signed in as user@example.com
  3. Login again         Re-authenticate with OAuth

  Press enter to confirm or esc to go back.
```

### API Key Inline Input

```
Enter Anthropic API Key

  Paste your key and press Enter. It will be stored securely.

  Key: sk-ant-******************|

  Press enter to save or esc to cancel.
```

### OAuth Sub-Menu

When user selects OAuth:

```
Select OAuth Method for Anthropic

> 1. Browser Login     Opens a URL, authenticate in browser
  2. Paste Token       Paste an OAuth access token directly

  Press enter to confirm or esc to go back.
```

Paste Token flow:

```
Enter Anthropic OAuth Token

  Paste your access token and press Enter.

  Token: ********************|

  Press enter to save or esc to cancel.
```

## /auth Command

Standalone command for managing credentials without switching models.

### Status View

```
/auth

+--------------------------------------------------+
|  Authentication Status                           |
|                                                  |
|  OpenAI:     OAuth (signed in as user@email)     |
|  Anthropic:  API Key (sk-ant-***dk3F)            |
|                                                  |
|  1. Manage OpenAI                                |
|  2. Manage Anthropic                             |
|                                                  |
|  Press enter to select or esc to dismiss.        |
+--------------------------------------------------+
```

### Manage Provider

Selecting a provider opens the same auth popup from the model-switch flow:

```
Manage Anthropic

> 1. API Key (current)     sk-ant-***dk3F
  2. Enter new API Key     Replace with a different key
  3. OAuth Login           Sign in with your Anthropic account
  4. Remove credentials    Delete stored Anthropic auth

  Press enter to confirm or esc to go back.
```

- Switching methods: pick a different one, old credentials stay until overwritten
- "Remove credentials": explicit destructive action with confirmation prompt "Remove all Anthropic credentials? (y/n)"
- No "logout" concept -- switch or explicitly remove

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Invalid API key format | Inline error: "Invalid key format. Anthropic keys start with sk-ant-". Retry without leaving popup. |
| OAuth token expired mid-session | Existing `refresh_anthropic_oauth_if_needed()` handles automatically. If refresh fails, error message: "Anthropic auth expired. Run /auth to re-authenticate." |
| API key rejected (401) | Existing 401 recovery state machine runs. If recovery fails: "Anthropic API key rejected. Run /auth to update." |
| Esc at auth step | Cancels entire model switch. Model stays on previous. No credentials changed. |
| Both API key and OAuth exist | `preferred_auth_modes` determines pre-highlight. Selecting the other updates preference. Both credential sets remain stored. |
| Env var vs stored credential | Auth popup shows both: "API Key (env var)" and "API Key (stored)" as separate options. Env var takes priority when both selected. |

## File Changes

### Core (auth storage -- minimal)

| File | Change |
|------|--------|
| `core/src/auth/storage.rs` | Add `preferred_auth_modes: HashMap<ProviderName, AuthMode>` to `AuthDotJsonV2`. Serialization/deserialization. |
| `core/src/auth/persistence.rs` | Read/write `preferred_auth_modes`. Merge-on-save preserves preferences. |

### TUI (main implementation)

| File | Change |
|------|--------|
| `tui/src/chatwidget.rs` | New `open_auth_popup()`. Modify `apply_model_and_effort()` to insert auth step. New `on_slash_auth()`. |
| `tui/src/chatwidget/tests.rs` | Snapshot tests for auth popup variants. |
| `tui/src/bottom_pane/chat_composer.rs` | Register `/auth` as slash command. |
| `tui/src/slash_command.rs` | Add `Auth` variant. |

### TUI App Server (mirror per convention 54)

Same files as TUI, mirrored.

### Post-change

- Run `just write-config-schema` after storage type changes.

### No changes to

- Protocol (`Op`, `EventMsg`)
- `anthropic_bridge.rs`
- `client.rs`
- `app-server-protocol/`
