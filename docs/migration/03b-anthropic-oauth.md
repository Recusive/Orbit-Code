# Stage 3b: Anthropic OAuth + Provider-Scoped Auth Storage

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Anthropic OAuth (Claude Pro/Max subscription login via browser code-paste) and redesign auth storage to support multiple providers side-by-side. After this stage, users can log in to both OpenAI and Anthropic simultaneously and switch between them based on model selection.

**Depends on:** Stage 3a complete (Anthropic wire protocol + API key via env var).

**Scope:** Production-ready, end-to-end. No MVP/v1 shortcuts. Every edge case, error path, and migration scenario is handled. All audit findings from `reviews/03b-anthropic-oauth.audit.md` are resolved.

**Tech Stack:** Rust, `reqwest`, SHA-256 PKCE (reuse from `login/src/pkce.rs`), `webbrowser`.

---

## Spec References

| Reference | Path | What to learn |
|-----------|------|---------------|
| **Anthropic OAuth plugin** | `reference/opencode-anthropic-auth/index.mjs` | OAuth PKCE flow, token exchange/refresh, beta headers, tool name prefixing |
| **Auth type union** | `reference/opencode/packages/opencode/src/auth/index.ts` | `oauth { refresh, access, expires }` vs `api { key }` — per-provider |
| **Plugin auth interface** | `reference/opencode/packages/plugin/src/index.ts:34-145` | AuthHook structure, `method: "code"` for code-paste flow |
| **Codex OAuth reference** | `reference/opencode/packages/opencode/src/plugin/codex.ts` | PKCE helpers, browser OAuth server, device code — comparison pattern |

---

## Public Surface Changes

| Surface | Before (after 3a) | After (3b) |
|---------|--------------------|------------|
| `orbit-code login` | OpenAI API key / ChatGPT OAuth only | **+ `--provider anthropic` with OAuth and API key methods** |
| `orbit-code logout` | Wipes all auth | **+ `--provider` flag for selective logout** |
| Auth storage | Single-provider `auth.json` (OpenAI only) | **Provider-scoped v2 format: OpenAI + Anthropic coexist** |
| `config.toml` | `model_provider = "anthropic"` + env var | **+ Anthropic API key/OAuth stored persistently** |
| TUI onboarding | ChatGPT / API Key options only | **+ Provider picker → Anthropic sign-in options** |
| App-server protocol | `LoginAccountParams`: `ApiKey`, `Chatgpt`, `ChatgptAuthTokens` | **+ `AnthropicApiKey`, `AnthropicOAuth` variants (v2 only)** |
| `AuthMode` enum | `ApiKey`, `Chatgpt`, `ChatgptAuthTokens` | **+ `AnthropicApiKey`, `AnthropicOAuth`** |
| `ModelProviderInfo` | `requires_openai_auth: bool` | **Renamed to `requires_auth: bool` (provider-agnostic)** |

---

## Current State (What Must Change)

### Auth Storage (`core/src/auth/storage.rs`)
```rust
// CURRENT: Single-provider, OpenAI-shaped
pub struct AuthDotJson {
    pub auth_mode: Option<AuthMode>,
    pub openai_api_key: Option<String>,  // ← hardcoded field name
    pub tokens: Option<TokenData>,       // ← single token block
    pub last_refresh: Option<DateTime<Utc>>,
}
```

### AuthMode (`app-server-protocol/src/protocol/common.rs`)
```rust
// CURRENT: OpenAI-only
pub enum AuthMode {
    ApiKey,
    Chatgpt,
    ChatgptAuthTokens,
}
```

### CodexAuth (`core/src/auth.rs`)
```rust
// CURRENT: 3 variants, all OpenAI
pub enum CodexAuth {
    ApiKey(ApiKeyAuth),
    Chatgpt(ChatgptAuth),
    ChatgptAuthTokens(ChatgptAuthTokens),
}
```

### Internal AuthMode (`core/src/auth.rs`)
```rust
// CURRENT: 2 variants only
pub enum AuthMode {
    ApiKey,
    Chatgpt,
}
```

### Login v2 (`app-server-protocol/src/protocol/v2.rs`)
```rust
// CURRENT: OpenAI-only login variants
pub enum LoginAccountParams {
    ApiKey { api_key: String },
    Chatgpt,
    ChatgptAuthTokens { access_token, chatgpt_account_id, chatgpt_plan_type },
}
pub enum LoginAccountResponse {
    ApiKey {},
    Chatgpt { login_id, auth_url },
    ChatgptAuthTokens {},
}
```

### TUI Auth (`tui/src/onboarding/auth.rs` + `tui_app_server/src/onboarding/auth.rs`)
```rust
// CURRENT: ChatGPT-only states (identical in both TUI implementations)
pub(crate) enum SignInState {
    PickMode,
    ChatGptContinueInBrowser(ContinueInBrowserState),
    ChatGptDeviceCode(ContinueWithDeviceCodeState),
    ChatGptSuccessMessage,
    ChatGptSuccess,
    ApiKeyEntry(ApiKeyInputState),
    ApiKeyConfigured,
}
pub(crate) enum SignInOption {
    ChatGpt,
    DeviceCode,
    ApiKey,
}
```

### ModelProviderInfo (`core/src/model_provider_info.rs`)
```rust
// CURRENT: OpenAI-centric naming
pub struct ModelProviderInfo {
    // ...
    pub requires_openai_auth: bool,  // ← misleading name for multi-provider
    // ...
}
```

### Telemetry (`otel/src/lib.rs`)
```rust
// CURRENT: Two variants only
pub enum TelemetryAuthMode {
    ApiKey,
    Chatgpt,
}
```

### CLI Login (`cli/src/login.rs`)
```rust
// CURRENT: All functions hardcoded to ChatGPT/OpenAI
pub async fn login_with_chatgpt(...) -> std::io::Result<()>
pub async fn run_login_with_chatgpt(...) -> !
pub async fn run_login_with_api_key(...) -> !
pub async fn run_login_with_device_code(...) -> !
```

### Anthropic Client (`anthropic/src/client.rs`)
```rust
// CURRENT: API key only, no OAuth mode
// Line 65: extra_headers.insert("x-api-key", header_value(&api_key)?);
// No Authorization: Bearer path for OAuth tokens
```

---

## Reference Implementation Notes

Critical details from reading the reference implementations. These override any assumptions:

### From `reference/opencode-anthropic-auth/index.mjs`

| Detail | Value | Lines |
|--------|-------|-------|
| **Token endpoint content type** | `Content-Type: application/json` (NOT form-encoded) | 43-44 |
| **Exchange request includes `state`** | Extracted from `code#state` paste, sent in JSON body | 40, 48 |
| **Authorize URL includes `code=true`** | Query param signals code-paste mode | 16 |
| **OAuth required beta headers** | `oauth-2025-04-20`, `interleaved-thinking-2025-05-14` | 176-179 |
| **OAuth must delete `x-api-key`** | Remove before setting `Authorization: Bearer` | 190 |
| **Tool prefix: `mcp_`** | Applied to tool defs AND `tool_use` blocks in messages | 213-238 |
| **Response strip: `mcp_` regex** | `/"name"\s*:\s*"mcp_([^"]+)"/g` on raw SSE text | 290-293 |
| **URL query: `?beta=true`** | Appended to `/v1/messages` for OAuth requests | 260-262 |
| **User-agent header** | `claude-cli/2.1.2 (external, cli)` | 187-189 |
| **Create API key response field** | `raw_key` (NOT `api_key`) | 351 |
| **Create API key: no request body** | POST with auth header only (no JSON body) | 342-350 |

### From `reference/opencode/packages/opencode/src/auth/index.ts`

| Detail | Value |
|--------|-------|
| **Auth types** | `oauth { refresh, access, expires, accountId?, enterpriseUrl? }`, `api { key }`, `wellknown { key, token }` |
| **Per-provider storage** | `Auth.get(providerID)`, `Auth.set(key, info)`, `Auth.all()`, `Auth.remove(key)` |
| **OAuth expires** | Stored as absolute millisecond timestamp (we convert to seconds per CLAUDE.md) |

### From `reference/opencode/packages/plugin/src/index.ts`

| Detail | Value |
|--------|-------|
| **Auth method types** | `"oauth"` (with `authorize()`) or `"api"` (with optional `authorize()`) |
| **Code-paste method** | `method: "code"` with `callback(code: string)` |
| **Auto method** | `method: "auto"` with `callback()` (no code param) |

### From `reference/opencode/packages/opencode/src/plugin/codex.ts`

| Detail | Value |
|--------|-------|
| **OAuth callback timeout** | 5 minutes (we use 10 minutes for code-paste — longer because manual) |
| **Codex rewrite URL** | `https://chatgpt.com/backend-api/codex/responses` (OpenAI-specific, not applicable to Anthropic) |

---

## Audit v2 Findings Resolution

The following critical and recommended findings from the **second audit** (`reviews/03b-anthropic-oauth.audit.md` — "NEEDS REWORK" verdict) are resolved:

| Audit v2 # | Finding | Resolution |
|-------------|---------|------------|
| Critical 1 | `AuthManager` provider-fixed conflicts with shared-manager architecture | **`AuthManager` is now provider-agnostic.** Caches auth for ALL providers in `HashMap<ProviderName, Option<CodexAuth>>`. Exposes `auth_for_provider()`, `reload_provider()`, `logout_provider()`, `logout_all()`. Constructor signature unchanged — no caller changes needed. Provider resolved at request/thread boundary. |
| Critical 2 | Status and logout undefined for multi-provider | **Per-provider semantics defined everywhere.** `login status` shows both providers. `logout` accepts `--provider` flag (CLI) or `LogoutAccountParams.provider` (app-server). `logout_provider()` removes one; `logout_all()` removes everything. |
| Critical 3 | App-server read-side contract still OpenAI-shaped | **Updated end-to-end.** `GetAccountResponse.requires_openai_auth` → `requires_auth` (with alias). `Account` enum gains `AnthropicApiKey`/`AnthropicOAuth` variants. `GetAccountParams` gains optional `provider` field. `AccountUpdatedNotification` gains optional `provider` field. `app_server_session.rs` bootstrap mapping updated. `StatusAccountDisplay` extended. |
| Critical 4 | Onboarding diverges from runtime auth precedence | **Centralized `provider_has_usable_auth()` helper** checks managed login, env vars, config headers, and bearer tokens — mirrors exact runtime resolution order. Both TUI bootstraps use it. |
| Rec 1 | Login crate uses raw `reqwest::Client::new()` | **Documented:** implementation must reuse CA-aware HTTP client from `login/src/server.rs`. |
| Rec 2 | `SubmitOAuthCodeResponse::Expired` defined but never returned | **Fixed.** `OAuthLoginManager.take()` now distinguishes expired (returns `Expired`) from unknown (returns `NotFound`). |
| Rec 3 | Missing file list for compatibility consumers | **Added** `app_server_session.rs`, `app-server-client`, `app-server-test-client` to file list and verification plan. |
| Rec 4 | Lockfile/Bazel steps overstated | **Made conditional** — only runs if `Cargo.toml` dependencies actually changed. |

---

## Audit v1 Findings Resolution

The following critical and recommended findings from the **first audit** (original `reviews/03b-anthropic-oauth.audit.md`) are resolved:

| Audit # | Finding | Resolution |
|---------|---------|------------|
| Critical 1 | Hybrid `AuthDotJson` v1/v2 struct is fragile | **Separate types**: `AuthDotJsonV1` (legacy) and `AuthDotJsonV2` (new). Convert on load via `TryFrom`. |
| Critical 2 | `load_auth()` doesn't know about providers | **Add `provider: &ProviderName` parameter** to `load_auth()`. Detect v2 format and use provider-keyed lookup. |
| Critical 3 | Provider selection at `load_auth()` time unspecified | **Provider-agnostic manager with per-provider accessors**: `AuthManager` caches auth for ALL providers. Callers use `auth_for_provider(ProviderName)` at request/thread time. Constructor unchanged. |
| Critical 4 | `UnauthorizedRecovery::has_next()` only checks ChatGPT | **Add `AnthropicOAuth` arms** to `has_next()` and `next()` with reload → refresh → done steps. |
| Critical 5 | `refresh_if_stale()` only handles ChatGPT | **Add `AnthropicOAuth` arm** that checks `expires_at` and calls `anthropic_refresh_token()`. |
| Rec 1 | Provider keys are stringly typed | **Use `ProviderName` enum** as HashMap key with `serde(rename_all = "lowercase")`. |
| Rec 2 | `ProviderAuth::ApiKey` is ambiguous | **Split into `OpenAiApiKey` and `AnthropicApiKey`** variants, each knowing their header format. |
| Rec 3 | `AnthropicClient::stream()` has no OAuth mode | **Add `AnthropicAuth` enum** (`ApiKey(String)` / `BearerToken(String)`) and update header injection. |
| Rec 4 | `PickProvider` rendering not shown | **Full rendering code specified** in Task 7 with back-navigation (Esc). |
| Rec 5 | Code-paste timeout missing | **10-minute TTL** on pending logins with background cleanup task. |
| Rec 6 | `expires_at` in milliseconds violates convention | **Use seconds** per CLAUDE.md rule 32. |
| Rec 7 | `auth.rs` already 1463 LoC | **Extract `core/src/auth/anthropic.rs`** for Anthropic-specific types and refresh logic. |
| Rec 8 | Migration is one-way and silent | **Warning log on migration + keep v1 backup** on first v2 write. |

---

## Files Modified

### Auth storage redesign:
- `codex-rs/core/src/auth/storage.rs` — Add `AuthDotJsonV2`, `ProviderAuth`, `ProviderName`; migration from v1 to v2; update `AuthStorageBackend` trait
- `codex-rs/core/src/auth/storage_tests.rs` — Tests for v1→v2 migration, round-trip, selective provider CRUD
- `codex-rs/core/src/auth/anthropic.rs` — (new) Anthropic-specific auth types (`AnthropicOAuthAuth`, `AnthropicApiKeyAuth`) and refresh logic
- `codex-rs/core/src/auth.rs` — Add `AnthropicApiKey`/`AnthropicOAuth` to `CodexAuth` and internal `AuthMode`; add per-provider accessors to `AuthManager` (constructor unchanged); extend `refresh_if_stale()` and `UnauthorizedRecovery`

### Protocol (v2 only — do NOT change v1 types):
- `codex-rs/app-server-protocol/src/protocol/common.rs` — Add Anthropic variants to `AuthMode`
- `codex-rs/app-server-protocol/src/protocol/v2.rs` — Add Anthropic variants to `LoginAccountParams`/`LoginAccountResponse`; add `SubmitOAuthCodeParams`/`Response` for code-paste flow

### Anthropic client:
- `codex-rs/anthropic/src/client.rs` — Add `AnthropicAuth` enum; update `stream()` to accept it and set header accordingly (`x-api-key` vs `Authorization: Bearer`)

### Login:
- `codex-rs/login/src/lib.rs` — Add re-exports for Anthropic auth functions
- `codex-rs/login/src/anthropic.rs` — (new) Anthropic OAuth authorize/exchange/refresh/create-api-key

### CLI:
- `codex-rs/cli/src/main.rs` — Add `--provider` flag to `login` and `logout` subcommands
- `codex-rs/cli/src/login.rs` — Add `run_login_with_anthropic_oauth()`, `run_login_with_anthropic_api_key()`; update `run_logout()` for selective provider logout

### TUI (mirror changes in both implementations):
- `codex-rs/tui/src/onboarding/auth.rs` — Add `PickProvider` state, Anthropic sign-in states + code-paste UI, back-navigation
- `codex-rs/tui_app_server/src/onboarding/auth.rs` — Same changes

### App-server (including read-side consumers):
- `codex-rs/app-server/src/orbit_code_message_processor.rs` — Add `login_anthropic_oauth_v2()`, `login_anthropic_api_key_v2()`, `submit_oauth_code_v2()` handlers; pending login TTL cleanup; update `logout_v2()`, `get_account()`
- `codex-rs/tui_app_server/src/app_server_session.rs` — Update `bootstrap()` and `status_account_display_from_auth_mode()` for Anthropic account variants
- `codex-rs/app-server-client/` — Update any client-side Account/AuthMode consumers
- `codex-rs/app-server-test-client/` — Update test helpers for new protocol variants

### Provider config:
- `codex-rs/core/src/model_provider_info.rs` — Rename `requires_openai_auth` → `requires_auth`; update Anthropic provider to `requires_auth: true`

### Core dispatch:
- `codex-rs/core/src/client.rs` — Update `stream_anthropic_messages()` to resolve auth from `AuthManager`; add OAuth-mode tool prefixing and header merge; update `AuthRequestTelemetryContext::new()`

### Telemetry:
- `codex-rs/otel/src/lib.rs` — Add `AnthropicApiKey`, `AnthropicOAuth` to `TelemetryAuthMode`

---

### Task 1: Redesign Auth Storage (Provider-Scoped V2)

Migrate from single-provider `AuthDotJson` to provider-scoped v2 format. Use **separate types** for v1 and v2 (resolves Audit Critical #1).

- [ ] **Step 1: Define `ProviderName` enum and `ProviderAuth` types**

```rust
// storage.rs — new types

/// Strongly-typed provider identifier used as HashMap key.
/// Extensible via new variants (e.g., OpenRouter in stage 3c).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderName {
    OpenAI,
    Anthropic,
}

impl fmt::Display for ProviderName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OpenAI => write!(f, "openai"),
            Self::Anthropic => write!(f, "anthropic"),
        }
    }
}

/// Per-provider auth credential stored in v2 format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ProviderAuth {
    /// OpenAI API key.
    #[serde(rename = "openai_api_key")]
    OpenAiApiKey { key: String },

    /// ChatGPT OAuth (OpenAI-specific, preserves existing token format).
    #[serde(rename = "chatgpt")]
    Chatgpt {
        tokens: TokenData,
        last_refresh: Option<DateTime<Utc>>,
    },

    /// ChatGPT external auth tokens (app-server managed).
    #[serde(rename = "chatgpt_auth_tokens")]
    ChatgptAuthTokens {
        tokens: TokenData,
        last_refresh: Option<DateTime<Utc>>,
    },

    /// Anthropic API key (persisted, not just env var).
    #[serde(rename = "anthropic_api_key")]
    AnthropicApiKey { key: String },

    /// Anthropic OAuth (code-paste flow).
    #[serde(rename = "anthropic_oauth")]
    AnthropicOAuth {
        access_token: String,
        refresh_token: String,
        /// Unix timestamp in SECONDS when the access token expires.
        /// Per CLAUDE.md rule 32: timestamps are integer Unix seconds.
        expires_at: i64,
    },
}
```

- [ ] **Step 2: Define `AuthDotJsonV2` as a distinct type**

```rust
/// V2 auth storage — supports multiple providers side-by-side.
/// This is a SEPARATE type from AuthDotJson (v1) to avoid the fragile hybrid
/// struct problem identified in the audit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDotJsonV2 {
    /// Always 2 for v2 format.
    pub version: u32,

    /// Provider-keyed auth credentials.
    pub providers: HashMap<ProviderName, ProviderAuth>,
}

impl AuthDotJsonV2 {
    pub fn new() -> Self {
        Self {
            version: 2,
            providers: HashMap::new(),
        }
    }

    /// Get auth for a specific provider.
    pub fn provider_auth(&self, provider: ProviderName) -> Option<&ProviderAuth> {
        self.providers.get(&provider)
    }

    /// Set auth for a specific provider (does not affect other providers).
    pub fn set_provider_auth(&mut self, provider: ProviderName, auth: ProviderAuth) {
        self.providers.insert(provider, auth);
    }

    /// Remove auth for a specific provider.
    pub fn remove_provider_auth(&mut self, provider: ProviderName) -> Option<ProviderAuth> {
        self.providers.remove(&provider)
    }

    /// Check if any provider has stored credentials.
    pub fn has_any_auth(&self) -> bool {
        !self.providers.is_empty()
    }
}
```

- [ ] **Step 3: Add v1→v2 migration function**

```rust
/// Migrate legacy v1 AuthDotJson to v2 format.
/// Called once on load when v1 format is detected.
impl From<AuthDotJson> for AuthDotJsonV2 {
    fn from(v1: AuthDotJson) -> Self {
        let mut v2 = AuthDotJsonV2::new();

        match v1.auth_mode {
            Some(AuthMode::ApiKey) | None if v1.openai_api_key.is_some() => {
                if let Some(key) = v1.openai_api_key {
                    v2.set_provider_auth(
                        ProviderName::OpenAI,
                        ProviderAuth::OpenAiApiKey { key },
                    );
                }
            }
            Some(AuthMode::Chatgpt) => {
                if let Some(tokens) = v1.tokens {
                    v2.set_provider_auth(
                        ProviderName::OpenAI,
                        ProviderAuth::Chatgpt {
                            tokens,
                            last_refresh: v1.last_refresh,
                        },
                    );
                }
            }
            Some(AuthMode::ChatgptAuthTokens) => {
                if let Some(tokens) = v1.tokens {
                    v2.set_provider_auth(
                        ProviderName::OpenAI,
                        ProviderAuth::ChatgptAuthTokens {
                            tokens,
                            last_refresh: v1.last_refresh,
                        },
                    );
                }
            }
            None => {} // No auth stored
        }

        v2
    }
}
```

- [ ] **Step 4: Update `AuthStorageBackend` trait for v2**

The storage backends now read/write `AuthDotJsonV2`. On load, detect format:
- If `"version": 2` present → deserialize as `AuthDotJsonV2`
- Otherwise → deserialize as `AuthDotJson` (v1), convert to v2 in memory

On save, always write v2 format. On first v2 write, create a backup of the v1 file:
```rust
// In FileAuthStorage::save() — before overwriting
fn backup_v1_if_needed(&self) -> std::io::Result<()> {
    let auth_file = get_auth_file(&self.orbit_code_home);
    let backup_file = self.orbit_code_home.join("auth.v1.json.bak");
    if auth_file.exists() && !backup_file.exists() {
        // Check if current file is v1 format
        if let Ok(contents) = std::fs::read_to_string(&auth_file) {
            if !contents.contains("\"version\"") {
                std::fs::copy(&auth_file, &backup_file)?;
                tracing::warn!(
                    "Migrated auth.json from v1 to v2 format. \
                     Backup saved to auth.v1.json.bak. \
                     Older Orbit Code versions cannot read v2 format."
                );
            }
        }
    }
    Ok(())
}
```

Update the trait signature:
```rust
pub(super) trait AuthStorageBackend: Debug + Send + Sync {
    fn load(&self) -> std::io::Result<Option<AuthDotJsonV2>>;
    fn save(&self, auth: &AuthDotJsonV2) -> std::io::Result<()>;
    fn delete(&self) -> std::io::Result<bool>;
    /// Delete auth for a single provider, preserving others.
    fn delete_provider(&self, provider: ProviderName) -> std::io::Result<bool>;
}
```

The `delete_provider` method loads, removes the provider entry, and saves (or deletes the file if empty).

- [ ] **Step 5: Update all 4 storage backends**

`FileAuthStorage`, `KeyringAuthStorage`, `AutoAuthStorage`, `EphemeralAuthStorage` — all updated to use `AuthDotJsonV2`. The format-detection logic lives in a shared helper:

```rust
/// Deserialize auth from JSON, auto-detecting v1 vs v2 format.
fn deserialize_auth(json: &str) -> Result<AuthDotJsonV2, serde_json::Error> {
    // Try v2 first (has "version" field)
    if let Ok(v2) = serde_json::from_str::<AuthDotJsonV2>(json) {
        return Ok(v2);
    }
    // Fall back to v1 and convert
    let v1: AuthDotJson = serde_json::from_str(json)?;
    Ok(AuthDotJsonV2::from(v1))
}
```

- [ ] **Step 6: Tests**

```bash
cd codex-rs && cargo test -p orbit-code-core -- auth::storage
just fmt
```

Test cases:
- Load v1 file → auto-migrates to v2 in memory
- Save v2 file → writes v2 format
- Load v2 file → reads correctly
- v1 backup created on first v2 write
- Round-trip: save v2 → load v2 → identical
- `delete_provider("anthropic")` preserves OpenAI auth
- `delete_provider` on last provider → removes file
- Concurrent v1 and v2 (two processes): v1 reader ignores unknown fields safely
- Empty `providers` map → `has_any_auth()` returns false
- `ProviderName` serde: serializes as lowercase strings

```bash
git add codex-rs/core/src/auth/
git commit -m "Redesign auth storage to provider-scoped v2 format with v1 backward compat"
```

---

### Task 2: Extend CodexAuth, AuthMode, and Telemetry

Add Anthropic variants to all auth enums and update exhaustive matches throughout.

- [ ] **Step 1: Create `core/src/auth/anthropic.rs`**

Extract Anthropic-specific auth types and refresh logic into a new module (resolves Audit Rec #7 — `auth.rs` is 1463 LoC):

```rust
//! Anthropic-specific authentication types and token refresh logic.

use std::sync::Arc;
use super::storage::AuthStorageBackend;

/// Anthropic API key auth (persisted in auth storage).
#[derive(Debug, Clone)]
pub struct AnthropicApiKeyAuth {
    pub(crate) api_key: String,
}

impl AnthropicApiKeyAuth {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

/// Anthropic OAuth auth (code-paste flow, refreshable).
#[derive(Debug, Clone)]
pub struct AnthropicOAuthAuth {
    pub(crate) access_token: String,
    pub(crate) refresh_token: String,
    /// Unix timestamp in seconds when access_token expires.
    pub(crate) expires_at: i64,
    pub(crate) storage: Arc<dyn AuthStorageBackend>,
}

impl AnthropicOAuthAuth {
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    /// Returns true if the access token will expire within the given buffer (seconds).
    pub fn is_expiring_within(&self, buffer_seconds: i64) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.expires_at.saturating_sub(now) < buffer_seconds
    }
}

/// Buffer in seconds before expiry to trigger proactive refresh.
pub const ANTHROPIC_TOKEN_REFRESH_BUFFER_SECONDS: i64 = 300; // 5 minutes
```

- [ ] **Step 2: Add Anthropic variants to `CodexAuth` and internal `AuthMode`**

In `core/src/auth.rs`:

```rust
pub mod anthropic;

use crate::auth::anthropic::AnthropicApiKeyAuth;
use crate::auth::anthropic::AnthropicOAuthAuth;

pub enum CodexAuth {
    ApiKey(ApiKeyAuth),
    Chatgpt(ChatgptAuth),
    ChatgptAuthTokens(ChatgptAuthTokens),
    AnthropicApiKey(AnthropicApiKeyAuth),
    AnthropicOAuth(AnthropicOAuthAuth),
}

pub enum AuthMode {
    ApiKey,
    Chatgpt,
    AnthropicApiKey,
    AnthropicOAuth,
}
```

- [ ] **Step 3: Add Anthropic variants to `ApiAuthMode` (protocol)**

In `app-server-protocol/src/protocol/common.rs`:

```rust
pub enum AuthMode {
    ApiKey,
    Chatgpt,
    #[serde(rename = "chatgptAuthTokens")]
    ChatgptAuthTokens,
    #[serde(rename = "anthropicApiKey")]
    AnthropicApiKey,
    #[serde(rename = "anthropicOAuth")]
    AnthropicOAuth,
}
```

- [ ] **Step 4: Add Anthropic variants to `TelemetryAuthMode`**

In `otel/src/lib.rs`:

```rust
pub enum TelemetryAuthMode {
    ApiKey,
    Chatgpt,
    AnthropicApiKey,
    AnthropicOAuth,
}
```

- [ ] **Step 5: Update ALL exhaustive matches**

This is the highest-risk step. Every `match` on `CodexAuth`, `AuthMode`, `ApiAuthMode`, and `TelemetryAuthMode` must gain new arms. The compiler will catch them all — but the *behavior* of each arm matters:

| File | Location | New arm behavior |
|------|----------|------------------|
| `core/src/auth.rs` | `CodexAuth::api_auth_mode()` | `AnthropicApiKey => ApiAuthMode::AnthropicApiKey`, `AnthropicOAuth => ApiAuthMode::AnthropicOAuth` |
| `core/src/auth.rs` | `CodexAuth::auth_mode()` (internal) | `AnthropicApiKey => AuthMode::AnthropicApiKey`, `AnthropicOAuth => AuthMode::AnthropicOAuth` |
| `core/src/auth.rs` | `From<AuthMode> for TelemetryAuthMode` | `AnthropicApiKey => TelemetryAuthMode::AnthropicApiKey`, `AnthropicOAuth => TelemetryAuthMode::AnthropicOAuth` |
| `core/src/auth.rs` | `CodexAuth::is_chatgpt_auth()` | Return `false` for both Anthropic variants |
| `core/src/auth.rs` | `enforce_login_restrictions()` | Anthropic modes bypass OpenAI workspace restrictions (return `None` for any `ForcedLoginMethod` check — Anthropic auth is not subject to OpenAI org enforcement) |
| `core/src/auth.rs` | `PartialEq for CodexAuth` | Already uses `api_auth_mode()` comparison — works automatically |
| `core/src/client.rs` | `AuthRequestTelemetryContext::new()` | `AnthropicApiKey => "AnthropicApiKey"`, `AnthropicOAuth => "AnthropicOAuth"` |
| `core/src/auth.rs` | `CodexAuth::access_token()` | `AnthropicApiKey => api_key`, `AnthropicOAuth => access_token` |
| `core/src/auth.rs` | `CodexAuth::refresh_token()` | `AnthropicApiKey => None`, `AnthropicOAuth => Some(refresh_token)` |

- [ ] **Step 6: Add `load_auth_for_provider()` — provider-scoped credential loading**

```rust
/// Load auth credentials for a specific provider.
///
/// Resolution order (per provider):
/// 1. Environment variable (ORBIT_API_KEY for OpenAI, ANTHROPIC_API_KEY for Anthropic)
/// 2. Ephemeral store (for external tokens from app-server)
/// 3. Persistent storage (file/keyring/auto)
///
/// Returns None if no auth found for the given provider.
fn load_auth_for_provider(
    orbit_code_home: &Path,
    provider: ProviderName,
    enable_orbit_code_api_key_env: bool,
    auth_credentials_store_mode: AuthCredentialsStoreMode,
) -> std::io::Result<Option<CodexAuth>> {
    // 1. Check provider-specific env var
    match provider {
        ProviderName::OpenAI => {
            if enable_orbit_code_api_key_env {
                if let Some(key) = env_api_key() {
                    return Ok(Some(CodexAuth::ApiKey(ApiKeyAuth { api_key: key })));
                }
            }
        }
        ProviderName::Anthropic => {
            if let Ok(key) = env::var("ANTHROPIC_API_KEY") {
                if !key.is_empty() {
                    return Ok(Some(CodexAuth::AnthropicApiKey(
                        AnthropicApiKeyAuth::new(key),
                    )));
                }
            }
        }
    }

    // 2. Check ephemeral store
    let ephemeral = create_auth_storage(
        orbit_code_home.to_path_buf(),
        AuthCredentialsStoreMode::Ephemeral,
    );
    if let Some(v2) = ephemeral.load()? {
        if let Some(codex_auth) = codex_auth_from_provider_auth(&v2, provider, ephemeral.clone()) {
            return Ok(Some(codex_auth));
        }
    }

    // 3. Check persistent storage
    let storage = create_auth_storage(
        orbit_code_home.to_path_buf(),
        auth_credentials_store_mode,
    );
    if let Some(v2) = storage.load()? {
        if let Some(codex_auth) = codex_auth_from_provider_auth(&v2, provider, storage.clone()) {
            return Ok(Some(codex_auth));
        }
    }

    Ok(None)
}

/// Convert a ProviderAuth entry from storage to a CodexAuth instance.
///
/// Takes `storage` as a parameter because `AnthropicOAuthAuth` needs a handle
/// to persist refreshed tokens. The caller (load_auth_for_provider) already
/// has the storage handle from the load step.
fn codex_auth_from_provider_auth(
    v2: &AuthDotJsonV2,
    provider: ProviderName,
    storage: Arc<dyn AuthStorageBackend>,
) -> Option<CodexAuth> {
    let provider_auth = v2.provider_auth(provider)?;
    match provider_auth {
        ProviderAuth::OpenAiApiKey { key } => {
            Some(CodexAuth::ApiKey(ApiKeyAuth { api_key: key.clone() }))
        }
        ProviderAuth::Chatgpt { tokens, .. } => {
            // Build ChatgptAuth from tokens + storage (existing logic)
            // ...
        }
        ProviderAuth::ChatgptAuthTokens { tokens, .. } => {
            // Build ChatgptAuthTokens from tokens (existing logic)
            // ...
        }
        ProviderAuth::AnthropicApiKey { key } => {
            Some(CodexAuth::AnthropicApiKey(AnthropicApiKeyAuth::new(key.clone())))
        }
        ProviderAuth::AnthropicOAuth { access_token, refresh_token, expires_at } => {
            Some(CodexAuth::AnthropicOAuth(AnthropicOAuthAuth {
                access_token: access_token.clone(),
                refresh_token: refresh_token.clone(),
                expires_at: *expires_at,
                storage: storage.clone(),
            }))
        }
    }
}
```

- [ ] **Step 7: Redesign `AuthManager` as provider-agnostic (two parallel auth domains)**

**This is the most critical architectural change.** The existing `AuthManager` is shared across
the entire process (`Arc<AuthManager>`) — one instance in the TUI (`tui/src/lib.rs:614`), one
in the app-server (`app-server/src/message_processor.rs:203`), and passed into `ThreadManager`
(`core/src/thread_manager.rs:168`). Meanwhile, `thread/start` already accepts `model_provider`,
so a single app-server process can host OpenAI and Anthropic threads simultaneously.

**The manager must NOT be bound to a single provider.** Instead, it caches auth for ALL providers
and exposes per-provider accessors. Provider resolution happens at the request/thread boundary,
not at process startup.

```rust
pub struct AuthManager {
    orbit_code_home: PathBuf,
    enable_orbit_code_api_key_env: bool,
    auth_credentials_store_mode: AuthCredentialsStoreMode,

    /// Per-provider auth cache. Both OpenAI and Anthropic can be cached simultaneously.
    cached_auth: RwLock<HashMap<ProviderName, Option<CodexAuth>>>,

    // Existing fields (external refresher, login state, etc.) remain unchanged
    external_auth_refresher: Option<Arc<dyn ExternalAuthRefresher>>,
    // ...
}

impl AuthManager {
    /// Constructor signature UNCHANGED from current code — no provider param.
    /// Callers (tui/lib.rs, message_processor.rs) do not change.
    pub fn new(
        orbit_code_home: PathBuf,
        enable_orbit_code_api_key_env: bool,
        auth_credentials_store_mode: AuthCredentialsStoreMode,
    ) -> Self {
        // Eagerly load OpenAI auth (preserves existing behavior)
        let openai_auth = load_auth_for_provider(
            &orbit_code_home,
            ProviderName::OpenAI,
            enable_orbit_code_api_key_env,
            auth_credentials_store_mode,
        ).ok().flatten();

        // Eagerly load Anthropic auth
        let anthropic_auth = load_auth_for_provider(
            &orbit_code_home,
            ProviderName::Anthropic,
            enable_orbit_code_api_key_env,
            auth_credentials_store_mode,
        ).ok().flatten();

        let mut cache = HashMap::new();
        cache.insert(ProviderName::OpenAI, openai_auth);
        cache.insert(ProviderName::Anthropic, anthropic_auth);

        Self {
            orbit_code_home,
            enable_orbit_code_api_key_env,
            auth_credentials_store_mode,
            cached_auth: RwLock::new(cache),
            // ...
        }
    }

    /// Existing `shared()` convenience method — signature unchanged.
    pub fn shared(...) -> Arc<Self> {
        Arc::new(Self::new(...))
    }

    // ─── Per-provider accessors ───

    /// Get cached auth for a specific provider, refreshing if stale.
    pub async fn auth_for_provider(&self, provider: ProviderName) -> Option<CodexAuth> {
        let auth = self.auth_cached_for_provider(provider);
        if let Some(ref a) = auth {
            let _ = self.refresh_if_stale(a, provider).await;
        }
        self.auth_cached_for_provider(provider)
    }

    /// Get cached auth for a specific provider without refresh.
    pub fn auth_cached_for_provider(&self, provider: ProviderName) -> Option<CodexAuth> {
        self.cached_auth.read().ok()
            .and_then(|cache| cache.get(&provider).cloned().flatten())
    }

    /// Reload auth for a specific provider from storage.
    pub async fn reload_provider(&self, provider: ProviderName) -> std::io::Result<()> {
        let auth = load_auth_for_provider(
            &self.orbit_code_home,
            provider,
            self.enable_orbit_code_api_key_env,
            self.auth_credentials_store_mode,
        )?;
        if let Ok(mut cache) = self.cached_auth.write() {
            cache.insert(provider, auth);
        }
        Ok(())
    }

    /// Logout a specific provider (remove from storage, clear cache).
    pub fn logout_provider(&self, provider: ProviderName) -> std::io::Result<bool> {
        let storage = create_auth_storage(
            self.orbit_code_home.clone(),
            self.auth_credentials_store_mode,
        );
        let removed = storage.delete_provider(provider)?;
        if let Ok(mut cache) = self.cached_auth.write() {
            cache.insert(provider, None);
        }
        Ok(removed)
    }

    /// Logout all providers (delete entire auth file, clear all cache).
    pub fn logout_all(&self) -> std::io::Result<bool> {
        let storage = create_auth_storage(
            self.orbit_code_home.clone(),
            self.auth_credentials_store_mode,
        );
        let removed = storage.delete()?;
        if let Ok(mut cache) = self.cached_auth.write() {
            for (_, v) in cache.iter_mut() {
                *v = None;
            }
        }
        Ok(removed)
    }

    // ─── Backward-compatible existing API ───

    /// Existing `auth()` method — returns OpenAI auth by default.
    /// Preserves backward compatibility for all existing callers that
    /// only know about OpenAI auth.
    pub async fn auth(&self) -> Option<CodexAuth> {
        self.auth_for_provider(ProviderName::OpenAI).await
    }

    /// Existing `auth_cached()` — returns OpenAI auth by default.
    pub fn auth_cached(&self) -> Option<CodexAuth> {
        self.auth_cached_for_provider(ProviderName::OpenAI)
    }

    /// Existing `reload()` — reloads all providers.
    pub async fn reload(&self) -> std::io::Result<()> {
        self.reload_provider(ProviderName::OpenAI).await?;
        self.reload_provider(ProviderName::Anthropic).await?;
        Ok(())
    }

    /// Existing `logout()` — logs out all providers (backward compat).
    pub fn logout(&self) -> std::io::Result<bool> {
        self.logout_all()
    }
}
```

**Key design decisions:**
- Constructor signature is UNCHANGED — `tui/src/lib.rs:614` and `message_processor.rs:203` don't change.
- `auth()` and `auth_cached()` default to OpenAI — all existing callers work without modification.
- New callers (Anthropic dispatch in `client.rs`) use `auth_for_provider(ProviderName::Anthropic)`.
- `ThreadManager` already receives `Arc<AuthManager>` — threads resolve their provider at request time via `auth_for_provider()`.
- Both auth domains are cached simultaneously. No startup-time binding.

- [ ] **Step 8: Add `provider_has_usable_auth()` — centralized auth detection**

Resolves Audit v2 Critical #4 (onboarding diverges from runtime). One helper that mirrors
the real runtime precedence, used by both TUI bootstraps and onboarding:

```rust
/// Check whether a provider has usable auth from ANY source.
///
/// This is a presence-only check — it returns true if ANY valid credential
/// exists, regardless of source. The check order does not matter because
/// this function never picks between sources; it just answers "can the
/// runtime succeed in authenticating for this provider?"
///
/// Sources checked:
/// - Managed login (stored in auth.json or keyring)
/// - Environment variable (ANTHROPIC_API_KEY, ORBIT_API_KEY)
/// - Provider config headers (http_headers.x-api-key)
/// - experimental_bearer_token
pub fn provider_has_usable_auth(
    provider_id: &str,
    provider_info: &ModelProviderInfo,
    auth_manager: Option<&AuthManager>,
) -> bool {
    let provider_name = provider_name_from_id(provider_id);

    // 1. Check managed auth
    if let Some(manager) = auth_manager {
        if manager.auth_cached_for_provider(provider_name).is_some() {
            return true;
        }
    }

    // 2. Check provider config headers (x-api-key from http_headers or env_http_headers)
    if provider_info.build_header_map()
        .ok()
        .and_then(|h| h.get("x-api-key").cloned())
        .is_some()
    {
        return true;
    }

    // 3. Check env var via provider config
    if provider_info.api_key().ok().flatten().is_some() {
        return true;
    }

    // 4. Check experimental bearer token
    if provider_info.experimental_bearer_token.is_some() {
        return true;
    }

    false
}

fn provider_name_from_id(id: &str) -> ProviderName {
    match id {
        "anthropic" => ProviderName::Anthropic,
        _ => ProviderName::OpenAI,
    }
}
```

- [ ] **Step 9: Update `refresh_if_stale()` for Anthropic OAuth**

The method now takes a `provider` param so it can reload the correct cache entry:

```rust
async fn refresh_if_stale(
    &self,
    auth: &CodexAuth,
    provider: ProviderName,
) -> Result<bool, RefreshTokenError> {
    match auth {
        CodexAuth::Chatgpt(chatgpt_auth) => {
            // ... existing ChatGPT refresh logic (unchanged) ...
        }
        CodexAuth::AnthropicOAuth(anthropic_auth) => {
            if !anthropic_auth.is_expiring_within(ANTHROPIC_TOKEN_REFRESH_BUFFER_SECONDS) {
                return Ok(false);
            }
            let tokens = orbit_code_login::anthropic_refresh_token(
                &anthropic_auth.refresh_token,
            )
            .await
            .map_err(|e| RefreshTokenError::Transient(std::io::Error::other(e)))?;

            let now = chrono::Utc::now().timestamp();
            let new_expires_at = now + i64::try_from(tokens.expires_in)
                .unwrap_or(3600);

            let storage = &anthropic_auth.storage;
            if let Some(mut v2) = storage.load()? {
                v2.set_provider_auth(
                    ProviderName::Anthropic,
                    ProviderAuth::AnthropicOAuth {
                        access_token: tokens.access_token,
                        refresh_token: tokens.refresh_token,
                        expires_at: new_expires_at,
                    },
                );
                storage.save(&v2)?;
            }

            self.reload_provider(provider).await?;
            Ok(true)
        }
        CodexAuth::ApiKey(_)
        | CodexAuth::ChatgptAuthTokens(_)
        | CodexAuth::AnthropicApiKey(_) => Ok(false),
    }
}
```

- [ ] **Step 10: Update `UnauthorizedRecovery` for Anthropic**

Recovery now tracks which provider it's recovering for. The constructor takes `provider`:

```rust
impl UnauthorizedRecovery {
    pub fn new(manager: Arc<AuthManager>, provider: ProviderName) -> Self {
        Self {
            manager,
            provider,
            step: UnauthorizedRecoveryStep::Reload,
        }
    }

    pub fn has_next(&self) -> bool {
        let auth = self.manager.auth_cached_for_provider(self.provider);
        let is_recoverable = auth.as_ref().is_some_and(|a| {
            a.is_chatgpt_auth()
                || matches!(a, CodexAuth::AnthropicOAuth(_))
        });
        if !is_recoverable {
            return false;
        }
        !matches!(self.step, UnauthorizedRecoveryStep::Done)
    }

    pub async fn next(&mut self) -> Result<(), UnauthorizedRecoveryError> {
        let auth = self.manager.auth_cached_for_provider(self.provider);
        match auth.as_ref() {
            Some(CodexAuth::AnthropicOAuth(_)) => {
                match self.step {
                    UnauthorizedRecoveryStep::Reload => {
                        self.manager.reload_provider(self.provider).await?;
                        self.step = UnauthorizedRecoveryStep::RefreshToken;
                        Ok(())
                    }
                    UnauthorizedRecoveryStep::RefreshToken => {
                        // Refresh the Anthropic token specifically
                        let auth = self.manager.auth_cached_for_provider(self.provider);
                        if let Some(CodexAuth::AnthropicOAuth(ref oauth)) = auth {
                            match self.manager.refresh_if_stale(
                                &CodexAuth::AnthropicOAuth(oauth.clone()),
                                self.provider,
                            ).await {
                                Ok(_) => {
                                    self.step = UnauthorizedRecoveryStep::Done;
                                    Ok(())
                                }
                                Err(e) => {
                                    self.step = UnauthorizedRecoveryStep::Done;
                                    Err(UnauthorizedRecoveryError::RefreshFailed(e))
                                }
                            }
                        } else {
                            self.step = UnauthorizedRecoveryStep::Done;
                            Err(UnauthorizedRecoveryError::NoMoreSteps)
                        }
                    }
                    _ => {
                        self.step = UnauthorizedRecoveryStep::Done;
                        Err(UnauthorizedRecoveryError::NoMoreSteps)
                    }
                }
            }
            Some(auth) if auth.is_chatgpt_auth() => {
                // ... existing ChatGPT recovery (unchanged) ...
            }
            _ => {
                self.step = UnauthorizedRecoveryStep::Done;
                Err(UnauthorizedRecoveryError::NoMoreSteps)
            }
        }
    }
}
```

- [ ] **Step 11: Format + lint + test**

```bash
just fmt
just fix -p orbit-code-core
just fix -p orbit-code-app-server-protocol
just fix -p orbit-code-otel
cargo test -p orbit-code-core -- auth
cargo test -p orbit-code-app-server-protocol
git add codex-rs/core/ codex-rs/app-server-protocol/ codex-rs/otel/
git commit -m "Add Anthropic auth modes with provider-agnostic AuthManager (two parallel auth domains)"
```

---

### Task 3: Add Anthropic OAuth Flow to Login Crate

Implement the OAuth PKCE flow for Claude Pro/Max subscription users.

**Spec reference:** `reference/opencode-anthropic-auth/index.mjs` lines 8-66 (authorize + exchange) and lines 312-354 (auth methods).

The flow is "code paste" — user opens browser, authorizes, sees a code, pastes it into the CLI. This is different from the existing ChatGPT OAuth which uses a local HTTP server callback.

- [ ] **Step 1: Create `login/src/anthropic.rs`**

**CRITICAL: The Anthropic token endpoint uses `Content-Type: application/json` with a JSON body,
NOT form-encoded.** The exchange request also includes the `state` field extracted from the
`code#state` paste. These details come from `reference/opencode-anthropic-auth/index.mjs`.

```rust
//! Anthropic OAuth flows for Claude Pro/Max subscription login.
//!
//! Supports two OAuth paths:
//! - MaxSubscription: claude.ai OAuth for Pro/Max subscribers
//! - ConsoleApiKey: console.anthropic.com OAuth for creating permanent API keys
//!
//! Both use PKCE code challenge with "code paste" flow (user copies code from browser).
//!
//! Reference implementation: reference/opencode-anthropic-auth/index.mjs

use crate::pkce::generate_pkce;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;
use url::Url;

// IMPLEMENTATION RULE: All HTTP calls in this module (token exchange,
// token refresh, API key creation) MUST use the same CA-aware reqwest
// client construction as login/src/server.rs. Do NOT use raw
// reqwest::Client::new(). The existing login crate provides a client
// builder that respects custom CA certificates and enterprise proxy
// configuration. Reuse it so Anthropic OAuth works in the same
// environments where OpenAI OAuth already works.
//
// The code below uses Client::new() for readability — replace with the
// shared CA-aware builder during implementation.

const ANTHROPIC_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const ANTHROPIC_TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";
const ANTHROPIC_REDIRECT_URI: &str = "https://console.anthropic.com/oauth/code/callback";
const ANTHROPIC_SCOPES: &str = "org:create_api_key user:profile user:inference";
const ANTHROPIC_CREATE_API_KEY_URL: &str =
    "https://api.anthropic.com/api/oauth/claude_cli/create_api_key";

#[derive(Debug, Clone, Copy)]
pub enum AnthropicAuthMode {
    /// Claude Pro/Max subscription via claude.ai
    MaxSubscription,
    /// Console API key creation via console.anthropic.com
    ConsoleApiKey,
}

/// Result of a successful token exchange or refresh.
#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicTokens {
    pub access_token: String,
    pub refresh_token: String,
    /// Token lifetime in seconds.
    pub expires_in: u64,
}

#[derive(Debug, Error)]
pub enum AnthropicLoginError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Token exchange failed: {message}")]
    TokenExchange { message: String },
    #[error("API key creation failed: {message}")]
    ApiKeyCreation { message: String },
    #[error("Invalid authorization code format")]
    InvalidCode,
}

/// Generate an authorization URL and PKCE verifier for the Anthropic OAuth flow.
///
/// The authorize URL includes `code=true` to signal code-paste mode.
/// State is set to the PKCE verifier (matches Anthropic reference implementation).
///
/// Returns `(auth_url, code_verifier)`.
pub fn anthropic_authorize_url(mode: AnthropicAuthMode) -> (String, String) {
    let pkce = generate_pkce();
    let base = match mode {
        AnthropicAuthMode::MaxSubscription => "https://claude.ai",
        AnthropicAuthMode::ConsoleApiKey => "https://console.anthropic.com",
    };

    let mut url = Url::parse(&format!("{base}/oauth/authorize"))
        .expect("static base URL is valid");

    url.query_pairs_mut()
        .append_pair("code", "true")  // Signals code-paste mode
        .append_pair("client_id", ANTHROPIC_CLIENT_ID)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", ANTHROPIC_REDIRECT_URI)
        .append_pair("scope", ANTHROPIC_SCOPES)
        .append_pair("code_challenge", &pkce.code_challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("state", &pkce.code_verifier); // State = verifier (matches spec)

    (url.to_string(), pkce.code_verifier)
}

/// Exchange the pasted authorization code for tokens.
///
/// The pasted input is in `{code}#{state}` format. Both parts are sent to the
/// token endpoint. Whitespace is trimmed.
///
/// **IMPORTANT:** The Anthropic token endpoint expects `Content-Type: application/json`,
/// NOT `application/x-www-form-urlencoded`. This differs from most OAuth implementations.
/// Reference: `reference/opencode-anthropic-auth/index.mjs` lines 39-66.
pub async fn anthropic_exchange_code(
    code_with_state: &str,
    verifier: &str,
) -> Result<AnthropicTokens, AnthropicLoginError> {
    let trimmed = code_with_state.trim();
    let (code, state) = match trimmed.split_once('#') {
        Some((c, s)) => (c, Some(s.to_string())),
        None => (trimmed, None),
    };

    if code.is_empty() {
        return Err(AnthropicLoginError::InvalidCode);
    }

    // Build JSON body — the token endpoint requires JSON, not form-encoded
    #[derive(Serialize)]
    struct ExchangeRequest {
        code: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        state: Option<String>,
        grant_type: String,
        client_id: String,
        redirect_uri: String,
        code_verifier: String,
    }

    let client = Client::new();
    let resp = client
        .post(ANTHROPIC_TOKEN_URL)
        .json(&ExchangeRequest {
            code: code.to_string(),
            state,
            grant_type: "authorization_code".to_string(),
            client_id: ANTHROPIC_CLIENT_ID.to_string(),
            redirect_uri: ANTHROPIC_REDIRECT_URI.to_string(),
            code_verifier: verifier.to_string(),
        })
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AnthropicLoginError::TokenExchange {
            message: format!("HTTP {status}: {body}"),
        });
    }

    resp.json::<AnthropicTokens>().await.map_err(Into::into)
}

/// Refresh an expired access token using a refresh token.
///
/// Also uses JSON body (same as exchange).
pub async fn anthropic_refresh_token(
    refresh_token: &str,
) -> Result<AnthropicTokens, AnthropicLoginError> {
    #[derive(Serialize)]
    struct RefreshRequest {
        grant_type: String,
        refresh_token: String,
        client_id: String,
    }

    let client = Client::new();
    let resp = client
        .post(ANTHROPIC_TOKEN_URL)
        .json(&RefreshRequest {
            grant_type: "refresh_token".to_string(),
            refresh_token: refresh_token.to_string(),
            client_id: ANTHROPIC_CLIENT_ID.to_string(),
        })
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AnthropicLoginError::TokenExchange {
            message: format!("HTTP {status}: {body}"),
        });
    }

    resp.json::<AnthropicTokens>().await.map_err(Into::into)
}

/// Exchange an OAuth access token for a permanent API key.
/// Only valid for ConsoleApiKey flow.
///
/// The response field is `raw_key` (not `api_key`).
/// Reference: `reference/opencode-anthropic-auth/index.mjs` line 351.
pub async fn anthropic_create_api_key(
    access_token: &str,
) -> Result<String, AnthropicLoginError> {
    #[derive(Deserialize)]
    struct CreateKeyResponse {
        raw_key: String,
    }

    let client = Client::new();
    let resp = client
        .post(ANTHROPIC_CREATE_API_KEY_URL)
        .header("Content-Type", "application/json")
        .header("authorization", format!("Bearer {access_token}"))
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AnthropicLoginError::ApiKeyCreation {
            message: format!("HTTP {status}: {body}"),
        });
    }

    let key_resp: CreateKeyResponse = resp.json().await?;
    Ok(key_resp.raw_key)
}
```

- [ ] **Step 2: Add to login crate exports**

```rust
// login/src/lib.rs — add these:
pub mod anthropic;
pub use anthropic::AnthropicAuthMode;
pub use anthropic::AnthropicLoginError;
pub use anthropic::AnthropicTokens;
pub use anthropic::anthropic_authorize_url;
pub use anthropic::anthropic_create_api_key;
pub use anthropic::anthropic_exchange_code;
pub use anthropic::anthropic_refresh_token;
```

- [ ] **Step 3: Add `url` dependency to login crate if not present**

Check `login/Cargo.toml` for `url` crate. Add if missing.

- [ ] **Step 4: Format + lint + test**

```bash
just fmt
just fix -p orbit-code-login
cargo test -p orbit-code-login
git add codex-rs/login/
git commit -m "Add Anthropic OAuth PKCE flow to login crate"
```

---

### Task 4: Update Anthropic Client for OAuth Mode

Add support for OAuth bearer tokens in addition to API keys (resolves Audit Rec #3).

- [ ] **Step 1: Define `AnthropicAuth` enum in `anthropic/src/client.rs`**

```rust
/// Authentication mode for Anthropic API requests.
#[derive(Debug, Clone)]
pub enum AnthropicAuth {
    /// API key: sent as `x-api-key` header.
    ApiKey(String),
    /// OAuth bearer token: sent as `Authorization: Bearer` header.
    /// When using OAuth:
    /// - Tool names must be prefixed with `mcp_`
    /// - `anthropic-beta` header must include `oauth-2025-04-20`
    /// - URL must include `?beta=true` query parameter
    BearerToken(String),
}
```

- [ ] **Step 2: Update `AnthropicClient::stream()` signature and header logic**

Change `api_key: String` parameter to `auth: AnthropicAuth`:

```rust
pub async fn stream(
    &self,
    request: MessagesRequest,
    auth: AnthropicAuth,      // Changed from: api_key: String
    mut extra_headers: HeaderMap,
) -> Result<impl Stream<Item = Result<AnthropicEvent, AnthropicError>>, AnthropicError> {
    // Set auth header based on mode
    // Reference: opencode-anthropic-auth/index.mjs lines 184-190
    match &auth {
        AnthropicAuth::ApiKey(key) => {
            extra_headers.insert("x-api-key", header_value(key)?);
        }
        AnthropicAuth::BearerToken(token) => {
            // OAuth mode: set Bearer token and REMOVE x-api-key if present
            extra_headers.insert(
                "authorization",
                header_value(&format!("Bearer {token}"))?,
            );
            extra_headers.remove("x-api-key");
            // Set user-agent for OAuth mode
            extra_headers.insert(
                "user-agent",
                HeaderValue::from_static("orbit-code-cli/1.0.0 (external, cli)"),
            );
        }
    }

    // Existing version/beta header logic...
    extra_headers.entry("anthropic-version")
        .or_insert(HeaderValue::from_static(ANTHROPIC_VERSION_HEADER_VALUE));
    extra_headers.entry("anthropic-beta")
        .or_insert(HeaderValue::from_static(ANTHROPIC_BETA_HEADER_VALUE));

    // If OAuth mode, merge required beta headers
    // Reference: opencode-anthropic-auth/index.mjs lines 176-182
    if matches!(auth, AnthropicAuth::BearerToken(_)) {
        merge_beta_header(&mut extra_headers, "oauth-2025-04-20");
        // interleaved-thinking is already in default beta headers
    }

    // Build URL — add ?beta=true for OAuth
    // Reference: opencode-anthropic-auth/index.mjs lines 257-267
    let mut url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
    if matches!(auth, AnthropicAuth::BearerToken(_)) {
        url.push_str("?beta=true");
    }

    // ... rest unchanged, using `url` instead of hardcoded path ...
}
```

- [ ] **Step 3: Update all callers of `AnthropicClient::stream()`**

In `core/src/client.rs` `stream_anthropic_messages()`:
```rust
// Replace:
//   client.stream(request, api_key, extra_headers)
// With:
//   client.stream(request, anthropic_auth, extra_headers)
// Where anthropic_auth is resolved from AuthManager (see Task 6)
```

- [ ] **Step 4: Add OAuth-mode tool name prefixing and stripping**

Tool name prefixing is required in THREE places for OAuth mode
(reference: `opencode-anthropic-auth/index.mjs` lines 213-238):

1. Tool definitions in `request.tools[].name`
2. `tool_use` content blocks in `request.messages[].content[]` (message history)
3. Response stream: strip `mcp_` from tool names in incoming SSE events

```rust
// In anthropic_bridge.rs (alongside existing translation functions)

const OAUTH_TOOL_PREFIX: &str = "mcp_";

/// Prefix tool names with `mcp_` for OAuth mode (Anthropic requirement).
/// Must prefix BOTH tool definitions AND tool_use blocks in message history.
pub fn prefix_tool_names_for_oauth(request: &mut MessagesRequest) {
    // 1. Prefix tool definitions
    for tool in &mut request.tools {
        if !tool.name.starts_with(OAUTH_TOOL_PREFIX) {
            tool.name = format!("{OAUTH_TOOL_PREFIX}{}", tool.name);
        }
    }

    // 2. Prefix tool_use blocks in message history
    for message in &mut request.messages {
        for content in &mut message.content {
            if let Content::ToolUse { name, .. } = content {
                if !name.starts_with(OAUTH_TOOL_PREFIX) {
                    *name = format!("{OAUTH_TOOL_PREFIX}{name}");
                }
            }
        }
    }
}

/// Strip `mcp_` prefix from tool names in responses (Anthropic requirement).
/// Applied to parsed events in the response stream mapper.
pub fn strip_oauth_tool_prefix(name: &str) -> &str {
    name.strip_prefix(OAUTH_TOOL_PREFIX).unwrap_or(name)
}
```

The stripping happens in `map_anthropic_to_response_stream()` — when `is_oauth` is true,
tool call names are stripped via `strip_oauth_tool_prefix()` before emitting
`ResponseItem::FunctionCall { name, .. }` events.

- [ ] **Step 5: Format + lint + test**

```bash
just fmt
just fix -p orbit-code-anthropic
cargo test -p orbit-code-anthropic
git add codex-rs/anthropic/
git commit -m "Add OAuth bearer token support to AnthropicClient"
```

---

### Task 5: Update App-Server Protocol (v2 Only)

Add Anthropic login variants to v2 protocol. **Do NOT modify v1 types.**

- [ ] **Step 1: Extend `LoginAccountParams` and `LoginAccountResponse`**

```rust
// app-server-protocol/src/protocol/v2.rs

pub enum LoginAccountParams {
    // existing...
    #[serde(rename = "apiKey", rename_all = "camelCase")]
    ApiKey { api_key: String },
    #[serde(rename = "chatgpt")]
    Chatgpt,
    #[experimental("account/login/start.chatgptAuthTokens")]
    #[serde(rename = "chatgptAuthTokens", rename_all = "camelCase")]
    ChatgptAuthTokens { ... },

    // NEW
    #[serde(rename = "anthropicApiKey", rename_all = "camelCase")]
    #[ts(rename = "anthropicApiKey", rename_all = "camelCase")]
    AnthropicApiKey {
        #[serde(rename = "apiKey")]
        #[ts(rename = "apiKey")]
        api_key: String,
    },

    #[serde(rename = "anthropicOAuth")]
    #[ts(rename = "anthropicOAuth")]
    AnthropicOAuth,
}

pub enum LoginAccountResponse {
    // existing...
    #[serde(rename = "apiKey", rename_all = "camelCase")]
    ApiKey {},
    #[serde(rename = "chatgpt", rename_all = "camelCase")]
    Chatgpt { login_id: String, auth_url: String },
    #[serde(rename = "chatgptAuthTokens", rename_all = "camelCase")]
    ChatgptAuthTokens {},

    // NEW
    #[serde(rename = "anthropicApiKey")]
    #[ts(rename = "anthropicApiKey")]
    AnthropicApiKey {},

    /// Code-paste flow: client must present auth_url to user,
    /// then submit the pasted code via SubmitOAuthCode.
    #[serde(rename = "anthropicOAuth", rename_all = "camelCase")]
    #[ts(rename = "anthropicOAuth", rename_all = "camelCase")]
    AnthropicOAuth {
        login_id: String,
        auth_url: String,
        /// Human-readable instruction for the user.
        instructions: String,
    },
}
```

- [ ] **Step 2: Add `Account::Anthropic` variants and update read-side contract**

Resolves Audit v2 Critical #3 — the read-side contract must represent Anthropic auth
just as cleanly as OpenAI auth.

```rust
// Account enum — add Anthropic variants
pub enum Account {
    #[serde(rename = "apiKey", rename_all = "camelCase")]
    ApiKey {},

    #[serde(rename = "chatgpt", rename_all = "camelCase")]
    Chatgpt { email: String, plan_type: PlanType },

    // NEW
    #[serde(rename = "anthropicApiKey", rename_all = "camelCase")]
    #[ts(rename = "anthropicApiKey", rename_all = "camelCase")]
    AnthropicApiKey {},

    #[serde(rename = "anthropicOAuth", rename_all = "camelCase")]
    #[ts(rename = "anthropicOAuth", rename_all = "camelCase")]
    AnthropicOAuth {},
}

// GetAccountResponse — rename requires_openai_auth to requires_auth
// with backward-compat alias
pub struct GetAccountResponse {
    pub account: Option<Account>,
    /// Whether this provider requires managed authentication.
    /// Renamed from requires_openai_auth for multi-provider support.
    #[serde(alias = "requires_openai_auth")]
    pub requires_auth: bool,
}

// GetAccountParams — add optional provider field
pub struct GetAccountParams {
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub refresh_token: bool,
    /// Provider to get account for. If omitted, returns account for the
    /// active thread's provider (or OpenAI by default).
    #[ts(optional = nullable)]
    pub provider: Option<String>,
}
```

Also update `AccountUpdatedNotification` to carry provider context:

```rust
pub struct AccountUpdatedNotification {
    pub auth_mode: Option<AuthMode>,
    pub plan_type: Option<PlanType>,
    /// Which provider this update applies to.
    #[ts(optional = nullable)]
    pub provider: Option<String>,
}
```

- [ ] **Step 3: Add `SubmitOAuthCode` request/response for code-paste flow**

```rust
/// Submit a pasted OAuth authorization code (used by Anthropic code-paste flow).
/// RPC method: `account/submitOAuthCode`
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "v2/")]
pub struct SubmitOAuthCodeParams {
    pub login_id: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export_to = "v2/")]
pub enum SubmitOAuthCodeResponse {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "failed")]
    Failed { message: String },
    #[serde(rename = "notFound")]
    NotFound,
    #[serde(rename = "expired")]
    Expired,
}
```

- [ ] **Step 4: Register in `ClientRequest` dispatch macro**

Add `SubmitOAuthCode` to the v2 request dispatch alongside existing `LoginAccount` and `CancelLoginAccount`.

- [ ] **Step 5: Regenerate schemas + test**

```bash
just write-app-server-schema
just write-app-server-schema --experimental
cargo test -p orbit-code-app-server-protocol
just fmt
git add codex-rs/app-server-protocol/
git commit -m "Add Anthropic login variants and SubmitOAuthCode to v2 protocol"
```

---

### Task 6: Wire Anthropic Auth into Core Dispatch

Update `client.rs` to resolve Anthropic auth from `AuthManager` instead of env var only.

- [ ] **Step 1: Update `stream_anthropic_messages()` in `client.rs`**

```rust
async fn stream_anthropic_messages(&self, ...) -> Result<ResponseStream> {
    // 1. Resolve Anthropic auth from AuthManager (stored credentials)
    let anthropic_auth = self.resolve_anthropic_auth()?;

    // 2. If OAuth mode, prefix tool names in request (tools + message history)
    let is_oauth = matches!(anthropic_auth, AnthropicAuth::BearerToken(_));
    if is_oauth {
        prefix_tool_names_for_oauth(&mut request);
    }

    // 3. Build headers (existing logic for version + beta headers)
    let mut extra_headers = provider.build_header_map()?;
    merge_anthropic_beta_headers(
        &mut extra_headers,
        model,
        &defaults.additional_beta_headers,
    )?;

    // 4. Create client and stream
    let client = AnthropicClient::new(...);
    let stream = client.stream(request, anthropic_auth, extra_headers).await?;

    // 5. Map response stream — pass is_oauth so tool names get mcp_ prefix stripped
    //    Reference: opencode-anthropic-auth/index.mjs lines 276-303
    map_anthropic_to_response_stream(stream, is_oauth)
    // When is_oauth=true, the mapper calls strip_oauth_tool_prefix() on
    // FunctionCall names before emitting ResponseItem::FunctionCall events
}

/// Resolve Anthropic authentication from AuthManager, falling back to env var.
///
/// Precedence:
/// 1. AuthManager (stored OAuth or API key from login)
/// 2. Provider headers (x-api-key from config http_headers)
/// 3. ANTHROPIC_API_KEY env var
/// 4. provider.experimental_bearer_token
fn resolve_anthropic_auth(&self) -> Result<AnthropicAuth> {
    // Check AuthManager for Anthropic-specific auth (provider-agnostic manager)
    if let Some(manager) = self.client.state.auth_manager.as_ref() {
        if let Some(auth) = manager.auth_cached_for_provider(ProviderName::Anthropic) {
            match auth {
                CodexAuth::AnthropicOAuth(ref oauth) => {
                    return Ok(AnthropicAuth::BearerToken(oauth.access_token().to_string()));
                }
                CodexAuth::AnthropicApiKey(ref api_key_auth) => {
                    return Ok(AnthropicAuth::ApiKey(api_key_auth.api_key().to_string()));
                }
                _ => {} // Unexpected variant — fall through
            }
        }
    }

    // Fall back to provider config headers or env var (existing 3a behavior)
    let provider = &self.client.state.provider;
    let extra_headers = provider.build_header_map()?;

    if let Some(api_key) = extra_headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .or(provider.api_key()?)
    {
        return Ok(AnthropicAuth::ApiKey(api_key));
    }

    if let Some(bearer) = &provider.experimental_bearer_token {
        return Ok(AnthropicAuth::BearerToken(bearer.clone()));
    }

    Err(CodexErr::EnvVar(EnvVarError {
        var: "ANTHROPIC_API_KEY",
        instructions: provider.env_key_instructions.as_deref(),
    }))
}
```

- [ ] **Step 2: Update `AuthRequestTelemetryContext::new()` match**

```rust
// Add arms for new auth modes
CodexAuth::AnthropicApiKey(_) => "AnthropicApiKey",
CodexAuth::AnthropicOAuth(_) => "AnthropicOAuth",
```

- [ ] **Step 3: Format + lint + test**

```bash
just fmt
just fix -p orbit-code-core
cargo test -p orbit-code-core
git add codex-rs/core/
git commit -m "Wire stored Anthropic auth into core dispatch with OAuth tool prefixing"
```

---

### Task 7: Update App-Server Login Handlers

- [ ] **Step 1: Add pending login state with TTL**

Resolves Audit Rec #5 — pending OAuth code-paste entries get a 10-minute timeout:

```rust
// In orbit_code_message_processor.rs

struct PendingOAuthLogin {
    verifier: String,
    created_at: std::time::Instant,
}

const PENDING_LOGIN_TTL: Duration = Duration::from_secs(600); // 10 minutes

impl OAuthLoginManager {
    fn insert(&mut self, login_id: String, verifier: String) {
        self.cleanup_expired();
        self.pending.insert(login_id, PendingOAuthLogin {
            verifier,
            created_at: std::time::Instant::now(),
        });
    }

    /// Returns Ok(verifier) if found and not expired, Err(true) if expired,
    /// Err(false) if never existed.
    fn take(&mut self, login_id: &str) -> Result<String, bool> {
        match self.pending.remove(login_id) {
            Some(p) if p.created_at.elapsed() < PENDING_LOGIN_TTL => Ok(p.verifier),
            Some(_) => Err(true),   // expired
            None => Err(false),      // unknown
        }
    }

    fn cleanup_expired(&mut self) {
        self.pending.retain(|_, v| v.created_at.elapsed() < PENDING_LOGIN_TTL);
    }
}
```

- [ ] **Step 2: Add handlers**

```rust
async fn login_anthropic_api_key_v2(
    &mut self,
    request_id: ConnectionRequestId,
    api_key: String,
) {
    // Save to provider-scoped auth storage under ProviderName::Anthropic
    let storage = self.auth_storage();
    let mut v2 = storage.load()?.unwrap_or_else(AuthDotJsonV2::new);
    v2.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicApiKey { key: api_key },
    );
    storage.save(&v2)?;

    // Reload AuthManager
    self.auth_manager.reload().await?;

    // Respond
    self.respond(request_id, LoginAccountResponse::AnthropicApiKey {});
}

async fn login_anthropic_oauth_v2(
    &mut self,
    request_id: ConnectionRequestId,
) {
    // Generate PKCE + auth URL
    let (auth_url, verifier) = anthropic_authorize_url(AnthropicAuthMode::MaxSubscription);
    let login_id = uuid::Uuid::new_v4().to_string();

    // Store pending login with TTL
    self.oauth_login_manager.insert(login_id.clone(), verifier);

    // Respond with URL for client to present
    self.respond(request_id, LoginAccountResponse::AnthropicOAuth {
        login_id,
        auth_url,
        instructions: "Open the URL above, authorize, then paste the code below.".to_string(),
    });
}

async fn submit_oauth_code_v2(
    &mut self,
    request_id: ConnectionRequestId,
    params: SubmitOAuthCodeParams,
) {
    // Look up pending login — distinguish expired from unknown
    let verifier = match self.oauth_login_manager.take(&params.login_id) {
        Ok(v) => v,
        Err(true) => {
            self.respond(request_id, SubmitOAuthCodeResponse::Expired);
            return;
        }
        Err(false) => {
            self.respond(request_id, SubmitOAuthCodeResponse::NotFound);
            return;
        }
    };

    // Exchange code for tokens
    match anthropic_exchange_code(&params.code, &verifier).await {
        Ok(tokens) => {
            let now = chrono::Utc::now().timestamp();
            let expires_at = now + i64::try_from(tokens.expires_in).unwrap_or(3600);

            // Save to provider-scoped auth storage
            let storage = self.auth_storage();
            let mut v2 = storage.load()?.unwrap_or_else(AuthDotJsonV2::new);
            v2.set_provider_auth(
                ProviderName::Anthropic,
                ProviderAuth::AnthropicOAuth {
                    access_token: tokens.access_token,
                    refresh_token: tokens.refresh_token,
                    expires_at,
                },
            );
            storage.save(&v2)?;

            // Reload AuthManager
            self.auth_manager.reload().await?;

            // Notify login complete
            self.notify(AccountLoginCompletedNotification {
                login_id: Some(params.login_id),
                success: true,
                error: None,
            });

            self.respond(request_id, SubmitOAuthCodeResponse::Success);
        }
        Err(e) => {
            self.respond(request_id, SubmitOAuthCodeResponse::Failed {
                message: e.to_string(),
            });
        }
    }
}
```

- [ ] **Step 3: Wire into `login_v2()` dispatch**

```rust
async fn login_v2(&mut self, request_id: ConnectionRequestId, params: LoginAccountParams) {
    match params {
        LoginAccountParams::ApiKey { api_key } => self.login_api_key_v2(request_id, api_key).await,
        LoginAccountParams::Chatgpt => self.login_chatgpt_v2(request_id).await,
        LoginAccountParams::ChatgptAuthTokens { .. } => { /* existing */ },
        LoginAccountParams::AnthropicApiKey { api_key } => {
            self.login_anthropic_api_key_v2(request_id, api_key).await
        }
        LoginAccountParams::AnthropicOAuth => {
            self.login_anthropic_oauth_v2(request_id).await
        }
    }
}
```

- [ ] **Step 4: Register `SubmitOAuthCode` as new v2 method**

Add `account/submitOAuthCode` to the v2 request dispatch.

- [ ] **Step 5: Update `logout_v2()` for provider-aware logout**

The current `logout_v2()` calls `self.auth_manager.logout()` which deletes everything.
Update to support per-provider logout:

```rust
// Add optional provider to LogoutAccountParams (v2 only):
pub struct LogoutAccountParams {
    /// Provider to log out from. If omitted, logs out from all providers.
    #[ts(optional = nullable)]
    pub provider: Option<String>,
}

async fn logout_v2(&mut self, request_id: ConnectionRequestId, params: LogoutAccountParams) {
    match params.provider.as_deref() {
        Some("anthropic") => {
            self.auth_manager.logout_provider(ProviderName::Anthropic)?;
        }
        Some("openai") => {
            self.auth_manager.logout_provider(ProviderName::OpenAI)?;
        }
        None => {
            self.auth_manager.logout_all()?;
        }
        Some(other) => {
            // Unknown provider — return error
        }
    }
    self.respond(request_id, LogoutAccountResponse {});
    self.broadcast_notification(self.current_account_updated_notification());
}
```

- [ ] **Step 6: Update `get_account()` for multi-provider**

The handler must be able to return account info for a specific provider:

```rust
async fn get_account(&self, request_id: ConnectionRequestId, params: GetAccountParams) {
    if params.refresh_token {
        // Refresh the relevant provider based on params.provider
        let provider = provider_name_from_id(
            params.provider.as_deref().unwrap_or(&self.default_provider_id())
        );
        let _ = self.auth_manager.reload_provider(provider).await;
    }

    // Omitted provider → config's default model_provider_id.
    // account/read is not thread-scoped; callers needing a specific
    // provider must pass it explicitly.
    let provider_id = params.provider.as_deref()
        .unwrap_or(&self.config.model_provider_id);
    let provider_name = provider_name_from_id(provider_id);

    let account = match self.auth_manager.auth_cached_for_provider(provider_name) {
        Some(CodexAuth::ApiKey(_)) => Some(Account::ApiKey {}),
        Some(CodexAuth::Chatgpt(ref chatgpt)) => Some(Account::Chatgpt {
            email: chatgpt.email().unwrap_or_default(),
            plan_type: chatgpt.plan_type(),
        }),
        Some(CodexAuth::AnthropicApiKey(_)) => Some(Account::AnthropicApiKey {}),
        Some(CodexAuth::AnthropicOAuth(_)) => Some(Account::AnthropicOAuth {}),
        _ => None,
    };

    self.respond(request_id, GetAccountResponse {
        account,
        requires_auth: self.config.model_provider.requires_auth,
    });
}
```

- [ ] **Step 7: Update `tui_app_server/src/app_server_session.rs` bootstrap mapping**

Resolves Audit v2 Critical #3 — the app-server-backed TUI bootstrap must understand
Anthropic account variants:

```rust
// In bootstrap() — add arms for Anthropic account types:
let (account_auth_mode, status_account_display) = match &account {
    Some(Account::ApiKey {}) => (
        Some(AuthMode::ApiKey),
        Some(StatusAccountDisplay::ApiKey),
    ),
    Some(Account::Chatgpt { email, plan_type }) => (
        Some(AuthMode::Chatgpt),
        Some(StatusAccountDisplay::ChatGpt { email: Some(email.clone()), plan: ... }),
    ),
    // NEW
    Some(Account::AnthropicApiKey {}) => (
        Some(AuthMode::AnthropicApiKey),
        Some(StatusAccountDisplay::AnthropicApiKey),
    ),
    Some(Account::AnthropicOAuth {}) => (
        Some(AuthMode::AnthropicOAuth),
        Some(StatusAccountDisplay::AnthropicOAuth),
    ),
    None => (None, None),
};

// Add to StatusAccountDisplay enum:
pub enum StatusAccountDisplay {
    ApiKey,
    ChatGpt { email: Option<String>, plan: String },
    AnthropicApiKey,
    AnthropicOAuth,
}
```

Also update `status_account_display_from_auth_mode()`:
```rust
fn status_account_display_from_auth_mode(mode: &AuthMode) -> Option<StatusAccountDisplay> {
    match mode {
        AuthMode::ApiKey => Some(StatusAccountDisplay::ApiKey),
        AuthMode::Chatgpt => Some(StatusAccountDisplay::ChatGpt { email: None, plan: ... }),
        AuthMode::AnthropicApiKey => Some(StatusAccountDisplay::AnthropicApiKey),
        AuthMode::AnthropicOAuth => Some(StatusAccountDisplay::AnthropicOAuth),
        _ => None,
    }
}
```

- [ ] **Step 8: Format + lint + test**

```bash
just fmt
just fix -p orbit-code-app-server
cargo test -p orbit-code-app-server
cargo test -p orbit-code-app-server-client
cargo test -p orbit-code-tui-app-server
git add codex-rs/app-server/ codex-rs/tui_app_server/
git commit -m "Add Anthropic login/logout/account handlers with provider-aware read-side contract"
```

---

### Task 8: Update CLI Login and Logout

- [ ] **Step 1: Add `--provider` flag to login subcommand**

In `cli/src/main.rs`:
```rust
struct LoginCommand {
    // existing flags...

    /// Provider to log in to. Defaults to openai.
    #[arg(long, value_parser = ["openai", "anthropic"])]
    provider: Option<String>,

    // ...
}
```

- [ ] **Step 2: Add Anthropic login functions to `cli/src/login.rs`**

```rust
pub async fn run_login_with_anthropic(
    cli_config_overrides: CliConfigOverrides,
) -> ! {
    println!("Anthropic Login");
    println!("───────────────");
    println!();
    println!("  1. Claude Pro/Max (OAuth — browser login)");
    println!("  2. Create API Key (OAuth → permanent key)");
    println!("  3. Enter API key manually");
    println!();

    let choice = prompt_selection(1..=3);

    match choice {
        1 => run_login_anthropic_oauth(cli_config_overrides).await,
        2 => run_login_anthropic_create_api_key(cli_config_overrides).await,
        3 => run_login_anthropic_api_key(cli_config_overrides).await,
        _ => unreachable!(),
    }
}

async fn run_login_anthropic_oauth(
    cli_config_overrides: CliConfigOverrides,
) -> ! {
    let (auth_url, verifier) = anthropic_authorize_url(AnthropicAuthMode::MaxSubscription);

    println!("Opening browser for authorization...");
    if webbrowser::open(&auth_url).is_err() {
        println!("Could not open browser. Please visit:");
        println!("  {auth_url}");
    }
    println!();
    println!("After authorizing, paste the code below:");

    let code = read_line_from_stdin();
    let tokens = anthropic_exchange_code(&code, &verifier)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Login failed: {e}");
            std::process::exit(1);
        });

    let now = chrono::Utc::now().timestamp();
    let expires_at = now + i64::try_from(tokens.expires_in).unwrap_or(3600);

    // Save to provider-scoped storage
    save_anthropic_oauth_auth(
        &cli_config_overrides,
        tokens.access_token,
        tokens.refresh_token,
        expires_at,
    );

    println!("Logged in to Anthropic via OAuth.");
    std::process::exit(0);
}

async fn run_login_anthropic_api_key(
    cli_config_overrides: CliConfigOverrides,
) -> ! {
    println!("Enter your Anthropic API key:");
    println!("  (Get one at https://console.anthropic.com/settings/keys)");

    let api_key = read_line_from_stdin();

    save_anthropic_api_key_auth(&cli_config_overrides, api_key.trim().to_string());

    println!("Anthropic API key saved.");
    std::process::exit(0);
}

async fn run_login_anthropic_create_api_key(
    cli_config_overrides: CliConfigOverrides,
) -> ! {
    let (auth_url, verifier) = anthropic_authorize_url(AnthropicAuthMode::ConsoleApiKey);

    println!("Opening browser for authorization...");
    if webbrowser::open(&auth_url).is_err() {
        println!("Could not open browser. Please visit:");
        println!("  {auth_url}");
    }
    println!();
    println!("After authorizing, paste the code below:");

    let code = read_line_from_stdin();
    let tokens = anthropic_exchange_code(&code, &verifier)
        .await
        .unwrap_or_else(|e| {
            eprintln!("Login failed: {e}");
            std::process::exit(1);
        });

    println!("Creating API key...");
    let api_key = anthropic_create_api_key(&tokens.access_token)
        .await
        .unwrap_or_else(|e| {
            eprintln!("API key creation failed: {e}");
            std::process::exit(1);
        });

    save_anthropic_api_key_auth(&cli_config_overrides, api_key);

    println!("Anthropic API key created and saved.");
    std::process::exit(0);
}
```

- [ ] **Step 3: Add `--provider` flag to logout**

```rust
struct LogoutCommand {
    /// Provider to log out from. If omitted, logs out from all providers.
    #[arg(long, value_parser = ["openai", "anthropic"])]
    provider: Option<String>,
}
```

Logout logic uses `AuthManager.logout_provider()` / `logout_all()`:
```rust
async fn run_logout(provider: Option<&str>, auth_manager: &AuthManager) -> ! {
    match provider {
        Some("anthropic") => {
            auth_manager.logout_provider(ProviderName::Anthropic)?;
            println!("Logged out from Anthropic.");
        }
        Some("openai") => {
            auth_manager.logout_provider(ProviderName::OpenAI)?;
            println!("Logged out from OpenAI.");
        }
        None => {
            auth_manager.logout_all()?;
            println!("Logged out from all providers.");
        }
        Some(other) => {
            eprintln!("Unknown provider: {other}");
            std::process::exit(1);
        }
    }
    std::process::exit(0);
}
```

- [ ] **Step 4: Wire new helper signatures in `main.rs`**

The new `run_login_status()` and `run_logout()` take `&AuthManager` instead of loading auth
directly. Update `cli/src/main.rs` to construct an `AuthManager` and pass it:

```rust
// In main.rs login subcommand dispatch:
let auth_manager = AuthManager::new(
    config.orbit_code_home.clone(),
    /*enable_orbit_code_api_key_env*/ true,
    config.cli_auth_credentials_store_mode,
);

match &login_cmd.action {
    Some(LoginSubcommand::Status) => {
        run_login_status(&auth_manager).await
    }
    _ if login_cmd.provider.as_deref() == Some("anthropic") => {
        run_login_with_anthropic(cli_config_overrides).await
    }
    _ => {
        // existing OpenAI login dispatch...
    }
}

// In main.rs logout subcommand dispatch:
match logout_cmd.provider.as_deref() {
    provider => run_logout(provider, &auth_manager).await,
}
```

- [ ] **Step 5: Update `run_login_status()` for multi-provider**

Resolves Audit v2 Critical #2 — status must show both providers:

```rust
pub async fn run_login_status(auth_manager: &AuthManager) -> ! {
    let mut found_any = false;

    // Check OpenAI
    if let Some(auth) = auth_manager.auth_cached_for_provider(ProviderName::OpenAI) {
        found_any = true;
        match auth {
            CodexAuth::ApiKey(ref a) => {
                println!("OpenAI: Logged in using an API key - {}", safe_format_key(a.api_key()));
            }
            CodexAuth::Chatgpt(_) | CodexAuth::ChatgptAuthTokens(_) => {
                println!("OpenAI: Logged in using ChatGPT");
            }
            _ => {}
        }
    }

    // Check Anthropic
    if let Some(auth) = auth_manager.auth_cached_for_provider(ProviderName::Anthropic) {
        found_any = true;
        match auth {
            CodexAuth::AnthropicApiKey(ref a) => {
                println!("Anthropic: Logged in using an API key - {}", safe_format_key(a.api_key()));
            }
            CodexAuth::AnthropicOAuth(_) => {
                println!("Anthropic: Logged in using OAuth");
            }
            _ => {}
        }
    }

    if !found_any {
        println!("Not logged in");
        std::process::exit(1);
    }
    std::process::exit(0);
}
```

- [ ] **Step 4: Format + lint + test**

```bash
just fmt
just fix -p orbit-code-cli
cargo test -p orbit-code-cli
git add codex-rs/cli/
git commit -m "Add --provider flag to CLI login/logout with Anthropic OAuth and API key flows"
```

---

### Task 9: Rename `requires_openai_auth` → `requires_auth`

Make the provider info field provider-agnostic. Update the Anthropic provider to require auth.

- [ ] **Step 1: Rename field in `ModelProviderInfo`**

In `core/src/model_provider_info.rs`:
```rust
pub struct ModelProviderInfo {
    // ...
    /// Does this provider require authentication?
    /// If true, user is presented with login screen on first run.
    #[serde(default, alias = "requires_openai_auth")] // backward compat for existing configs
    pub requires_auth: bool,
    // ...
}
```

The `alias` ensures existing `config.toml` files with `requires_openai_auth` still work.

- [ ] **Step 2: Update all usages**

Search for `requires_openai_auth` across the codebase and rename to `requires_auth`. Key locations:
- `core/src/model_provider_info.rs` — built-in provider definitions
- `tui/src/lib.rs` — onboarding decision
- `tui_app_server/src/lib.rs` — onboarding decision
- `tui/src/app.rs` — login status
- `tui_app_server/src/app.rs` — login status

- [ ] **Step 3: Update Anthropic provider to require auth**

```rust
// In create_anthropic_provider():
ModelProviderInfo {
    // ...
    requires_auth: true,  // Changed from false — Anthropic now has persistent login
    // ...
}
```

- [ ] **Step 4: Format + lint + test**

```bash
just fmt
just fix -p orbit-code-core
just fix -p orbit-code-tui
just fix -p orbit-code-tui-app-server
cargo test -p orbit-code-core
git add codex-rs/
git commit -m "Rename requires_openai_auth to requires_auth and enable for Anthropic provider"
```

---

### Task 10: Update TUI Onboarding (Both Implementations)

Changes must be mirrored in `tui/src/onboarding/auth.rs` AND `tui_app_server/src/onboarding/auth.rs`.

- [ ] **Step 1: Extend `SignInOption` and `SignInState`**

```rust
pub(crate) enum SignInOption {
    // existing
    ChatGpt,
    DeviceCode,
    ApiKey,
    // NEW — Anthropic options
    AnthropicOAuth,
    AnthropicApiKey,
}

pub(crate) enum SignInState {
    // existing
    PickMode,
    ChatGptContinueInBrowser(ContinueInBrowserState),
    ChatGptDeviceCode(ContinueWithDeviceCodeState),
    ChatGptSuccessMessage,
    ChatGptSuccess,
    ApiKeyEntry(ApiKeyInputState),
    ApiKeyConfigured,
    // NEW — provider picker first, then per-provider states
    PickProvider,                // "OpenAI" vs "Anthropic"
    AnthropicPickMethod,         // OAuth vs API Key
    AnthropicOAuthPasteCode(AnthropicOAuthState),
    AnthropicOAuthSuccess,
    AnthropicApiKeyEntry(ApiKeyInputState),
    AnthropicApiKeyConfigured,
}
```

- [ ] **Step 2: Add `AnthropicOAuthState`**

```rust
#[derive(Clone)]
pub(crate) struct AnthropicOAuthState {
    auth_url: String,
    verifier: String,
    input_value: String,       // user types/pastes code here
    cursor_position: usize,
    error: Option<String>,
}
```

- [ ] **Step 3: Implement `PickProvider` rendering**

Resolves Audit Rec #4 — full rendering and navigation for provider picker:

```rust
fn render_pick_provider(&self, area: Rect, buf: &mut Buffer) {
    // "Choose a provider:"
    // "  > OpenAI" (highlighted)
    // "    Anthropic"
    // "[↑↓] Navigate  [Enter] Select  [Esc] Back"

    let options = [
        ("OpenAI", "ChatGPT account or API key"),
        ("Anthropic", "Claude Pro/Max or API key"),
    ];
    // ... render with standard list selection pattern (matches PickMode style)
}

fn handle_pick_provider_input(&mut self, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Down => {
            // Toggle between OpenAI and Anthropic
        }
        KeyCode::Enter => {
            match self.highlighted_provider {
                0 => {
                    // Transition to PickMode (existing OpenAI flow)
                    *self.sign_in_state.write() = SignInState::PickMode;
                }
                1 => {
                    // Transition to AnthropicPickMethod
                    *self.sign_in_state.write() = SignInState::AnthropicPickMethod;
                }
                _ => {}
            }
        }
        KeyCode::Esc => {
            // Back to PickProvider (or exit if already at top)
        }
        _ => {}
    }
}
```

- [ ] **Step 4: Implement `AnthropicPickMethod` rendering**

```rust
fn render_anthropic_pick_method(&self, area: Rect, buf: &mut Buffer) {
    // "Anthropic Login"
    // "  > Claude Pro/Max (OAuth)"
    // "    Enter API key"
    // "[↑↓] Navigate  [Enter] Select  [Esc] Back"
}
```

- [ ] **Step 5: Implement `AnthropicOAuthPasteCode` rendering and input**

```rust
fn render_anthropic_oauth(&self, area: Rect, buf: &mut Buffer, state: &AnthropicOAuthState) {
    // "Opening browser for Anthropic authorization..."
    // ""
    // "If the browser didn't open, visit:"
    // "  https://claude.ai/oauth/authorize?..."  (cyan, underlined)
    // ""
    // "Paste the authorization code:"
    // "[__________________________]"  (input field)
    // ""
    // "[Enter] Submit  [Esc] Cancel"
    //
    // If state.error is Some:
    // "  Error: ..." (red)
}

fn handle_anthropic_oauth_input(&mut self, key: KeyEvent, state: &mut AnthropicOAuthState) {
    match key.code {
        KeyCode::Char(c) => {
            state.input_value.insert(state.cursor_position, c);
            state.cursor_position += 1;
        }
        KeyCode::Backspace => {
            if state.cursor_position > 0 {
                state.cursor_position -= 1;
                state.input_value.remove(state.cursor_position);
            }
        }
        KeyCode::Enter => {
            if !state.input_value.trim().is_empty() {
                // Spawn async task to exchange code
                let code = state.input_value.clone();
                let verifier = state.verifier.clone();
                // ... exchange code, on success -> AnthropicOAuthSuccess
                // ... on failure -> set state.error
            }
        }
        KeyCode::Esc => {
            // Back to AnthropicPickMethod
            *self.sign_in_state.write() = SignInState::AnthropicPickMethod;
        }
        _ => {}
    }
}
```

- [ ] **Step 6: Update onboarding gate to use centralized auth detection**

Resolves Audit v2 Critical #4 — onboarding must respect ALL runtime auth sources
(env vars, config headers, stored credentials), not just managed login.

In both TUI implementations (`tui/src/lib.rs` and `tui_app_server/src/lib.rs`), replace
the current `requires_openai_auth` check with `provider_has_usable_auth()`:

```rust
// BEFORE (tui/src/lib.rs ~line 632):
// if config.model_provider.requires_openai_auth && !has_auth { show_onboarding() }

// AFTER:
let needs_onboarding = config.model_provider.requires_auth
    && !provider_has_usable_auth(
        &config.model_provider_id,
        &config.model_provider,
        Some(&auth_manager),
    );
if needs_onboarding {
    show_onboarding();
}
```

When the onboarding widget does show, select initial state based on the active provider:
```rust
fn initial_sign_in_state(config: &Config) -> SignInState {
    match config.model_provider_id.as_str() {
        "anthropic" => SignInState::AnthropicPickMethod,
        _ => SignInState::PickMode,  // existing OpenAI flow
    }
}
```

For the app-server TUI variant, the gate logic is identical but works through the
bootstrap `GetAccountResponse.requires_auth` field and the `Account` variant.
If bootstrap returns `Account::AnthropicApiKey` or `Account::AnthropicOAuth`,
the user is already authenticated — skip onboarding.

- [ ] **Step 7: Snapshot review + commit**

```bash
just fmt
cargo test -p orbit-code-tui
cargo test -p orbit-code-tui-app-server
cargo insta pending-snapshots -p orbit-code-tui
cargo insta pending-snapshots -p orbit-code-tui-app-server
# Review and accept snapshots
cargo insta accept -p orbit-code-tui
cargo insta accept -p orbit-code-tui-app-server
git add codex-rs/tui/ codex-rs/tui_app_server/
git commit -m "Add Anthropic sign-in to TUI onboarding with provider picker (both implementations)"
```

---

### Task 11: Verify and Finalize

- [ ] **Step 1: Full workspace check + lint**

```bash
cd codex-rs && cargo check
just fmt
just fix -p orbit-code-anthropic
just fix -p orbit-code-core
just fix -p orbit-code-login
just fix -p orbit-code-cli
just fix -p orbit-code-app-server-protocol
just fix -p orbit-code-app-server
just fix -p orbit-code-tui
just fix -p orbit-code-tui-app-server
just fix -p orbit-code-otel
```

- [ ] **Step 2: Run all affected crate tests (including read-side consumers)**

```bash
cargo test -p orbit-code-anthropic
cargo test -p orbit-code-core
cargo test -p orbit-code-login
cargo test -p orbit-code-cli
cargo test -p orbit-code-app-server-protocol
cargo test -p orbit-code-app-server
cargo test -p orbit-code-app-server-client
cargo test -p orbit-code-tui
cargo test -p orbit-code-tui-app-server
cargo test -p orbit-code-otel
```

- [ ] **Step 3: Schema and snapshot regeneration**

```bash
just write-config-schema
just write-app-server-schema
just write-app-server-schema --experimental
cargo insta pending-snapshots -p orbit-code-tui
cargo insta pending-snapshots -p orbit-code-tui-app-server
```

- [ ] **Step 4: Lockfile and Bazel (conditional — only if dependencies changed)**

Only run these if `[workspace.dependencies]` in `Cargo.toml` actually changed:
```bash
# Check if Cargo.toml dependencies changed:
git diff --name-only | grep -q 'Cargo.toml' && {
    cargo generate-lockfile
    just bazel-lock-update
    just bazel-lock-check
}
```

- [ ] **Step 5: Final commit**

```bash
git add codex-rs/
git commit -m "Stage 3b complete: Anthropic OAuth + provider-scoped auth storage"
```

---

## Edge Cases Handled

All edge cases from the audit are addressed:

| Edge Case | Resolution |
|-----------|------------|
| **v1 `auth.json` with OpenAI-only auth** | Auto-migrated to v2 on load. v1 backup created on first v2 write. |
| **`auth.json` with both v1 fields AND `providers` map** | Impossible with separate types: `AuthDotJsonV2` has no legacy fields. If somehow present, `deserialize_auth()` tries v2 first, falls back to v1 conversion. |
| **`resolved_mode()` misclassifies Anthropic-only v2 files** | Eliminated: `resolved_mode()` is replaced by `codex_auth_from_provider_auth()` which uses provider-keyed lookup. No fallthrough to ChatGPT. |
| **User runs `orbit-code logout` without `--provider`** | Deletes entire auth file (all providers). With `--provider anthropic`, selectively removes Anthropic entry using `delete_provider()`. |
| **Anthropic OAuth refresh returns extra fields** | `AnthropicTokens` struct uses `serde(default)` — unknown fields are ignored. |
| **`ANTHROPIC_API_KEY` env var AND stored Anthropic OAuth coexist** | Env var takes precedence (checked first in `load_auth()`). Documented in the function. |
| **User has Anthropic OAuth but selects OpenAI model** | Auth selection is model-driven via `auth_for_provider()` at the request boundary. OpenAI model → `auth_for_provider(OpenAI)`. Anthropic model → `auth_for_provider(Anthropic)`. The shared `AuthManager` caches both; no cross-provider confusion. |
| **One app-server process hosts threads with different `model_provider`** | `AuthManager` is provider-agnostic and shared across all threads. Each thread resolves its provider at `thread/start` time via `ConfigOverrides.model_provider`, then calls `auth_for_provider()` with the correct `ProviderName`. Concurrent OpenAI + Anthropic threads work correctly. |
| **User has valid `ANTHROPIC_API_KEY` env var but no managed login** | `provider_has_usable_auth()` checks env vars via `provider.api_key()` — returns true, onboarding is skipped. Runtime dispatch in `stream_anthropic_messages()` falls through to env var. No unnecessary login flow. |
| **User has `http_headers.x-api-key` in config but no managed login** | `provider_has_usable_auth()` checks `build_header_map()` for `x-api-key` — returns true, onboarding is skipped. Runtime dispatch picks it up from provider headers. |
| **`codex login status` with both providers logged in** | Shows both: "OpenAI: Logged in using ..." and "Anthropic: Logged in using ...". Exit code 0 if any provider is logged in. |
| **`account/logout` with both providers logged in** | If `provider` param is set → `logout_provider()` removes only that one. If omitted → `logout_all()`. Notification includes provider context. |
| **Expired vs unknown `SubmitOAuthCode` login ID** | `OAuthLoginManager.take()` returns `Err(true)` for expired, `Err(false)` for unknown. Handler returns `SubmitOAuthCodeResponse::Expired` vs `::NotFound` accordingly. |
| **App-server TUI bootstrap receives Anthropic account** | `app_server_session.rs` bootstrap mapping updated with `Account::AnthropicApiKey` and `Account::AnthropicOAuth` arms. `StatusAccountDisplay` enum extended. |
| **`account/read` with both providers authenticated, caller omits `provider`** | Returns the account for the config's default `model_provider_id`. This is the process-level default, not a thread-level concept — `account/read` is not thread-scoped. Callers that need a specific provider must pass `provider` explicitly. |
| **`login status` with one provider via env var, other via managed login** | Both are shown. Env-var-only auth displays as "Logged in using API key (env)" to distinguish from managed login. `provider_has_usable_auth()` detects both. |
| **`logout_all()` notification behavior** | Emits ONE `AccountUpdatedNotification` with `provider: None` and `auth_mode: None`. Not one per provider — the "all" semantics mean the entire auth state changed atomically. Per-provider logout emits one notification with the specific `provider` set. |
| **Concurrent migration from multiple processes** | v2 format is always written atomically (truncate + write). v1 reader sees unknown `providers` field and ignores it (serde `default`). v2 reader works correctly. v1 backup prevents data loss. |
| **Code-paste timeout** | Pending logins expire after 10 minutes. Expired entries cleaned up lazily on next operation. `SubmitOAuthCodeResponse::Expired` returned for timed-out entries. |
| **User starts OAuth flow then cancels (Ctrl+C in TUI)** | Pending login state is in-memory only — cleaned up on process exit. No leaked state. |
| **Token expiry mid-session** | `refresh_if_stale()` checks `expires_at` with 5-minute buffer. If refresh fails, `UnauthorizedRecovery` attempts one more refresh before surfacing error. |
| **Downgrade to older binary** | Warning logged on v1→v2 migration. v1 backup preserved. Older binary reads v2 file, sees unknown fields, ignores them — existing OpenAI auth lost. Documented as breaking change with mitigation (backup). |
| **`ForcedLoginMethod` with Anthropic auth** | Anthropic auth modes bypass `ForcedLoginMethod` entirely — it only governs OpenAI login method (Api vs Chatgpt). No conflict. |

---

## Auth Precedence (Documented)

For each provider, auth is resolved in this order (highest wins):

### OpenAI
1. `ORBIT_API_KEY` or `CODEX_API_KEY` env var (if `enable_orbit_code_api_key_env`)
2. Ephemeral store (external ChatGPT tokens from app-server)
3. Persistent storage (file/keyring/auto) under `ProviderName::OpenAI`

### Anthropic
1. `ANTHROPIC_API_KEY` env var
2. Ephemeral store (if set by app-server)
3. Persistent storage under `ProviderName::Anthropic`
4. Provider config `http_headers` `x-api-key` (from `config.toml`)
5. Provider config `experimental_bearer_token`

---

## Expected Outcomes

After Stage 3b:

- Users can `orbit-code login --provider anthropic` and choose OAuth or API key
- OAuth flow: browser opens → user pastes code → tokens stored → automatic refresh
- API key flow: user enters key or creates one via OAuth → stored persistently
- OpenAI and Anthropic auth coexist in provider-scoped `auth.json` (v2 format)
- Existing v1 `auth.json` auto-migrated with backup
- TUI onboarding shows provider picker → Anthropic options
- App-server clients can log in to Anthropic via v2 protocol
- OAuth mode enables `mcp_` tool prefixing and Anthropic-specific beta headers
- Token refresh happens proactively (5 min buffer) and reactively (401 recovery)
- Selective logout via `--provider` flag
- All auth selection is model-driven — no manual switching
- All timestamps in seconds per CLAUDE.md convention

## What's Next

- **Stage 3c:** OpenRouter provider (Chat Completions API)
- **Stage 4:** Provider registry
- **Stage 5:** Auth system unification (if further generalization needed beyond 3b)
