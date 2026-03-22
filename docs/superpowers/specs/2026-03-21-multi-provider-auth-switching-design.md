# Multi-Provider Auth Switching (v3 — Final)

> **Date:** 2026-03-21
> **Revised:** 2026-03-21 (post-audit v3 — extend V2, don't replace)
> **Status:** Approved
> **Approach:** Extend V2 storage + TUI Phase 1

## Problem

Users cannot switch between auth methods (API key vs OAuth) or add credentials for a new provider mid-session. The V2 storage holds only one `ProviderAuth` per provider, so storing both an API key AND OAuth tokens for the same provider is impossible. The TUI has no flow to prompt for credentials when switching providers.

## Audit Findings Addressed (v2 + v3)

| Audit Issue | Resolution |
|-------------|------------|
| Storage only holds one ProviderAuth per provider | Extend V2 with `alternate_credentials` map (no format version change) |
| V3 format change ripples through 5+ writer sites | No format change. `save_auth`/`save_auth_v2` already centralize writes. New fields are backward-compatible `serde(default)`. Zero caller changes needed. |
| `tui_app_server` mirror ignores server-driven auth | Phase 1 scoped to standalone `tui` only. |
| Env var vs stored credential picker unsupported | Dropped. Env vars keep existing precedence. |
| "Paste Token" underspecified for OAuth | Dropped. OAuth = browser flow only. |
| Growing `chatwidget.rs` further | New `chatwidget/auth_popup.rs` submodule. |
| Tests pointed at nonexistent `persistence_tests.rs` | Tests go in existing `auth_tests.rs` and `storage_tests.rs`. |
| `just write-config-schema` not relevant | Removed. |
| V2 writers (onboarding, CLI, app-server, refresh) not updated | No update needed — they go through `save_auth`/`save_auth_v2` which auto-preserve new fields via load-merge-save. |
| No backup/rollback for format migration | `backup_before_alternate_write()` added, same pattern as `backup_v1_if_needed()`. |
| `to_v1_openai()` needs preferred_mode semantics | Uses `preferred_auth_modes` to pick which OpenAI credential to project to v1. |
| Downgrade safety with old binaries | No version change. Old binaries read V2, ignore new fields via `serde(default)`. |

## Design Decisions

| Decision | Choice |
|----------|--------|
| Storage approach | Extend V2 with optional fields, no version bump |
| When auth picker appears | Always when switching providers, even if credentials exist |
| Auth methods per provider | API key + OAuth for both OpenAI and Anthropic |
| Popup structure | Three-step sequential: model -> effort -> auth method |
| API key entry | Inline masked text input in TUI popup |
| OAuth entry | Browser login flow only (no token paste) |
| Auth persistence | `preferred_auth_modes` map + `alternate_credentials` map |
| Credential swap on switch | Active credential in `providers`, inactive in `alternate_credentials` |
| Standalone management | Dedicated `/auth` command |
| Scope | Phase 1: standalone `tui` only |

## Data Model

### Extended V2 (backward-compatible)

```rust
pub struct AuthDotJsonV2 {
    pub version: u32,  // stays 2

    /// Active credential per provider (existing, unchanged).
    pub providers: HashMap<ProviderName, ProviderAuth>,

    /// Stored-but-inactive credential per provider.
    /// When user switches auth method, the old credential moves here
    /// and the new one goes into `providers`.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub alternate_credentials: HashMap<ProviderName, ProviderAuth>,

    /// Last-used auth method per provider. Determines which credential
    /// is pre-highlighted in the auth popup.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub preferred_auth_modes: HashMap<ProviderName, AuthMode>,
}
```

### How the swap works

**The key insight: `set_provider_auth()` itself auto-preserves the old credential.**

```rust
pub fn set_provider_auth(&mut self, provider: ProviderName, auth: ProviderAuth) {
    if let Some(old) = self.providers.insert(provider, auth) {
        // Only preserve as alternate when the auth METHOD TYPE changes
        // (e.g., API key -> OAuth). Same-method rewrites (e.g., OAuth token
        // refresh, API key rotation) replace in place without touching alternate.
        if std::mem::discriminant(&old)
            != std::mem::discriminant(
                self.providers.get(&provider).expect("just inserted"),
            )
        {
            self.alternate_credentials.insert(provider, old);
        }
    }
}
```

The `discriminant` check means:
- **OAuth refresh** writes new `AnthropicOAuth` over old `AnthropicOAuth` → same discriminant → alternate (API key) untouched
- **API key rotation** writes new `AnthropicApiKey` over old `AnthropicApiKey` → same discriminant → alternate untouched
- **Method switch** writes `AnthropicOAuth` over `AnthropicApiKey` → different discriminant → old API key moves to alternate

ALL existing callers get correct behavior without any code changes.

When user switches from API key to OAuth for Anthropic:

```
Before:
  providers:              { Anthropic: AnthropicApiKey { key: "sk-ant-..." } }
  alternate_credentials:  {}

After (set_provider_auth automatically moves old to alternate):
  providers:              { Anthropic: AnthropicOAuth { access_token: ..., refresh_token: ..., expires_at: ... } }
  alternate_credentials:  { Anthropic: AnthropicApiKey { key: "sk-ant-..." } }
  preferred_auth_modes:   { Anthropic: AnthropicOAuth }
```

When user switches back to API key:

```
After:
  providers:              { Anthropic: AnthropicApiKey { key: "sk-ant-..." } }
  alternate_credentials:  { Anthropic: AnthropicOAuth { ... } }
  preferred_auth_modes:   { Anthropic: AnthropicApiKey }
```

The swap ensures `providers` always holds the active credential. Existing code that reads `providers` keeps working unchanged.

**Same-method rewrites are safe.** If Anthropic OAuth refresh writes a new token via `set_provider_auth()`, the discriminant matches (`AnthropicOAuth` -> `AnthropicOAuth`), so alternate credentials are untouched. The stored API key alternate survives refresh cycles.

**Cross-method switches are also safe.** If onboarding writes an Anthropic OAuth credential when an API key was active, the discriminant differs (`AnthropicApiKey` -> `AnthropicOAuth`), so the old API key auto-moves to alternate. No caller needs to know about `alternate_credentials`.

### Backward compatibility

- **Old binaries reading new file:** `serde(default)` means new fields default to empty HashMaps. Old binary sees only `providers` — works as before.
- **Old binaries writing new file:** Old binary's `save_auth`/`save_auth_v2` does load-merge-save. It loads the full struct (including new fields via `serde(default)`), merges its changes into `providers`, then serializes. Since the old struct definition doesn't include the new fields, they are lost on write. This is acceptable because:
  - Users don't run multiple binary versions simultaneously
  - `backup_before_alternate_write()` preserves the pre-switch state
  - The active credential in `providers` is always the correct one for old code
- **New binaries reading old file:** New fields default to empty. No migration needed.

### `auth_cached_for_provider()` changes

Minimal change — the method already loads from V2 storage. The only addition: when loading from storage for a provider, check `preferred_auth_modes` to decide whether to return the credential from `providers` or `alternate_credentials`:

```
auth_cached_for_provider(provider):
  // existing: check cached, check env vars
  // then load from v2 storage:
  load AuthDotJsonV2
  preferred = v2.preferred_auth_modes.get(provider)
  active = v2.providers.get(provider)
  alternate = v2.alternate_credentials.get(provider)

  // If preferred mode matches the alternate credential, use alternate
  // Otherwise use active (existing behavior)
```

In practice, the swap operation keeps `providers` aligned with `preferred_auth_modes`, so `alternate_credentials` is only used as a restoration source — not for request-time resolution. The resolution path stays simple.

### `to_v1_openai()` changes

When both API key and ChatGPT OAuth exist for OpenAI, use `preferred_auth_modes` to pick which one projects to V1 format. If no preference, use `providers` entry (existing behavior).

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
User picks effort -> determine target provider from model slug
  |-- Same provider as current? -> skip auth popup, apply immediately
  |-- Different provider?
       -> Load V2 storage for target provider
       -> Check providers + alternate_credentials
       -> Build auth popup items based on what exists
       -> User picks method
       -> If existing credential selected: swap if needed, update preferred_mode
       -> If new credential needed: show inline input or browser flow
       -> Save via save_auth_v2() (merge-on-save preserves everything)
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

### API key active, OAuth stored as alternate

```
Select Authentication for Anthropic

> 1. API Key (active)    sk-ant-*******dk3F
  2. Enter new API Key   Replace with a different key
  3. OAuth (stored)      Use stored OAuth tokens
  4. OAuth Login         Re-authenticate with OAuth

  Press enter to confirm or esc to go back.
```

### Only API key stored (no alternate)

```
Select Authentication for Anthropic

> 1. API Key (active)    sk-ant-*******dk3F
  2. Enter new API Key   Replace with a different key
  3. OAuth Login         Sign in with your Anthropic account

  Press enter to confirm or esc to go back.
```

### API Key Inline Input

```
Enter Anthropic API Key

  Paste your key and press Enter. It will be stored securely.

  Key: sk-ant-******************|

  Press enter to save or esc to cancel.
```

Key is masked as user types. Basic format validation on Enter:
- Anthropic: starts with `sk-ant-`
- OpenAI: starts with `sk-`
On failure: inline error, retry without leaving popup.

### OAuth Browser Flow

Selecting "OAuth Login" triggers the existing headless OAuth flow:
- OpenAI: existing `headless_chatgpt_login` in `tui/src/onboarding/auth/`
- Anthropic: existing Anthropic OAuth flow

No "paste token" option.

## /auth Command

### Status View

```
/auth

+--------------------------------------------------+
|  Authentication Status                           |
|                                                  |
|  OpenAI:     OAuth (active)                      |
|              API Key: sk-*****3kF (stored)       |
|                                                  |
|  Anthropic:  API Key (active): sk-ant-***dk3F    |
|              OAuth: not configured               |
|                                                  |
|  1. Manage OpenAI                                |
|  2. Manage Anthropic                             |
|                                                  |
|  Press enter to select or esc to dismiss.        |
+--------------------------------------------------+
```

Shows both active and alternate credentials per provider.

### Manage Provider

```
Manage Anthropic

> 1. API Key (active)     sk-ant-***dk3F
  2. Enter new API Key    Replace with a different key
  3. OAuth Login          Sign in with your Anthropic account
  4. Remove credentials   Delete all stored Anthropic auth

  Press enter to confirm or esc to go back.
```

- Switching methods: swaps between `providers` and `alternate_credentials`, updates `preferred_auth_modes`
- "Remove credentials": destructive, confirmation prompt, clears both `providers` and `alternate_credentials` entries for that provider

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Invalid API key format | Inline error, retry without leaving popup |
| OAuth token expired | Existing refresh logic. If fails: "Run /auth to re-authenticate." |
| API key rejected (401) | Existing 401 recovery. If fails: "Run /auth to update." |
| Esc at auth step | Cancels model switch. No credentials changed. |
| Both credentials exist | `preferred_auth_modes` determines pre-highlight. Active is in `providers`, stored is in `alternate_credentials`. |
| Remove credentials while env var set | Stored credentials removed. Env var still works via existing precedence. |
| Old binary overwrites after switch | `backup_before_alternate_write()` preserves state. Active credential in `providers` remains correct for old code. |

## File Changes

### Core (extend V2 — backward-compatible)

| File | Change |
|------|--------|
| `core/src/auth/storage.rs` | Add `alternate_credentials` and `preferred_auth_modes` fields to `AuthDotJsonV2`. Add swap helper methods. Add `backup_before_alternate_write()`. |
| `core/src/auth/storage_tests.rs` | Round-trip tests for new fields. Backward-compat deserialization test (V2 without new fields). Swap operation tests. |
| `core/src/auth/persistence.rs` | `save_auth_v2` merge logic includes new fields. Add `swap_auth_method()` helper. |
| `core/src/auth_tests.rs` | Tests for `preferred_mode` resolution in `auth_cached_for_provider`. |
| `core/src/auth/manager.rs` | `auth_cached_for_provider()` checks `preferred_auth_modes` + `alternate_credentials`. |

### TUI (Phase 1 — standalone only)

| File | Change |
|------|--------|
| `tui/src/slash_command.rs` | Add `Auth` variant. |
| `tui/src/chatwidget/auth_popup.rs` | NEW submodule: `open_auth_popup()`, `open_api_key_input()`, masked input, provider detection. |
| `tui/src/chatwidget.rs` | Wire auth step into model-switch. Wire `/auth` command. Delegate to `auth_popup` submodule. |
| `tui/src/chatwidget/tests.rs` | Snapshot tests for auth popups. |

### No changes needed (auto-preserved by `set_provider_auth()` swap)

These callers write credentials via `set_provider_auth()` (directly or through `save_auth`/`save_auth_v2`). Because `set_provider_auth()` now auto-moves the old credential to `alternate_credentials`, all of them preserve existing credentials without any code changes:

| File | Write path | Why safe |
|------|-----------|----------|
| `tui/src/onboarding/auth.rs` | `save_auth()` -> `set_provider_auth()` | Old credential auto-moved to alternate |
| `tui_app_server/src/onboarding/auth.rs` | Same path | Same reason. Phase 2 for UI changes. |
| `cli/src/login.rs` | `login_with_api_key()` -> `save_auth()` -> `set_provider_auth()` | Old credential auto-moved to alternate |
| `app-server/orbit_code_message_processor.rs` | `save_auth_v2()` -> `set_provider_auth()` | Old credential auto-moved to alternate |
| `core/src/anthropic_auth/refresh.rs` | `save_auth_v2()` -> `set_provider_auth()` | Old credential auto-moved to alternate |
| Protocol (`Op`, `EventMsg`) | No changes |
| `anthropic_bridge.rs` | No changes |
| `client.rs` | Uses `auth_cached_for_provider()` which we update |
| `app-server-protocol/` | No changes |
