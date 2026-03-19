# Stage 5: Auth System

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the existing `orbit-code-login` and `orbit-code-core/auth` subsystems to support multi-provider authentication -- API key storage per provider (env vars + config + persisted auth.json), OAuth device-code flows for Claude, GitHub Copilot, and GitLab, and auth provider selection in the TUI onboarding screen.

**Architecture:** The current auth system is OpenAI-specific: it supports `ApiKey` vs `Chatgpt` (OAuth) modes, stores tokens in `auth.json`, and the TUI onboarding presents "Log in with ChatGPT" or "Enter API key" options. We generalize this to a per-provider auth model where each provider has its own key/token stored independently, the auth resolution logic checks env vars first, then config file, then persisted tokens. The existing OpenAI OAuth flow continues working unchanged.

**Tech Stack:** Rust, `reqwest`, `tiny_http`, `webbrowser`, `sha2`, `serde_json`.

**Depends on:** Stage 4 (provider registry with `AuthType` and `env_keys` per provider).

---

## Reference: TypeScript Auth System

The TypeScript `auth/service.ts` stores auth info per provider in a JSON file:
```typescript
type Info = Oauth | Api | WellKnown
// Oauth: { type: "oauth", refresh, access, expires, accountId }
// Api: { type: "api", key }
// WellKnown: { type: "wellknown", key, token }
```

Key features:
- `canonicalKey()` normalizes provider IDs (e.g., `"opencode"` -> `"orbit"`)
- `relatedKeys()` handles aliases for the same provider
- Auth is stored per-provider in a single `auth.json` file
- OAuth uses refresh tokens with auto-renewal

---

## Files Modified

**Core auth module:**
- `codex-rs/core/src/auth.rs` -- Extend `AuthMode` and `CodexAuth` for multi-provider
- `codex-rs/core/src/auth/storage.rs` -- Extend `AuthDotJson` for per-provider storage

**Login crate:**
- `codex-rs/login/src/lib.rs` -- Add multi-provider exports
- `codex-rs/login/src/device_code_auth.rs` -- Generalize for non-OpenAI providers
- `codex-rs/login/src/server.rs` -- Generalize OAuth callback for multi-provider

**New files in login crate:**
- `codex-rs/login/src/anthropic_auth.rs` -- Claude console.anthropic.com API key flow
- `codex-rs/login/src/copilot_auth.rs` -- GitHub Copilot device code OAuth
- `codex-rs/login/src/gitlab_auth.rs` -- GitLab OAuth flow

**Config:**
- `codex-rs/core/src/config/mod.rs` -- Add `provider_auth` config key

**TUI onboarding:**
- `codex-rs/tui/src/onboarding/auth.rs` -- Multi-provider auth selection UI

---

### Task 1: Extend Auth Storage for Multi-Provider

**Files:**
- Modify: `codex-rs/core/src/auth.rs`
- Modify: `codex-rs/core/src/auth/storage.rs`

- [ ] **Step 1: Extend `AuthDotJson` for per-provider storage**

Currently `auth.json` stores a single set of credentials. Extend it to store per-provider auth keyed by provider ID:

```rust
/// Persisted auth state. The top-level keys are provider IDs.
/// Example:
/// {
///   "openai": { "type": "chatgpt", "access_token": "...", "refresh_token": "..." },
///   "anthropic": { "type": "api_key", "key": "sk-ant-..." },
///   "github-copilot": { "type": "oauth", "access_token": "...", "refresh_token": "..." }
/// }
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthDotJson {
    /// Legacy flat fields for backward compatibility with existing OpenAI auth.
    #[serde(flatten)]
    pub legacy: Option<LegacyAuthFields>,

    /// Per-provider auth entries.
    #[serde(flatten)]
    pub providers: HashMap<String, ProviderAuth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderAuth {
    ApiKey { key: String },
    OAuth {
        access_token: String,
        refresh_token: Option<String>,
        expires_at: Option<i64>,
        account_id: Option<String>,
    },
    Chatgpt {
        // Keep existing ChatGPT auth structure for backward compat
        access_token: String,
        refresh_token: String,
        expires_at: i64,
    },
}
```

- [ ] **Step 2: Implement migration from old auth.json format**

On startup, if `auth.json` contains the legacy flat format (non-provider-keyed), migrate it:
- Detect legacy format by checking for `access_token` / `api_key` at root level
- Wrap in `providers: { "openai": { ... } }`
- Write back the new format

- [ ] **Step 3: Add per-provider read/write methods**

```rust
impl AuthDotJson {
    /// Get auth for a specific provider.
    pub fn get_provider_auth(&self, provider_id: &str) -> Option<&ProviderAuth> { ... }

    /// Set auth for a specific provider.
    pub fn set_provider_auth(&mut self, provider_id: &str, auth: ProviderAuth) { ... }

    /// Remove auth for a specific provider.
    pub fn remove_provider_auth(&mut self, provider_id: &str) { ... }

    /// List all providers with stored auth.
    pub fn authenticated_providers(&self) -> Vec<&str> { ... }
}
```

- [ ] **Step 4: Commit**

```bash
git add codex-rs/core/src/auth.rs codex-rs/core/src/auth/storage.rs
git commit -m "Extend auth storage for per-provider credentials in auth.json"
```

---

### Task 2: Extend `AuthMode` and `CodexAuth` for Multi-Provider

**Files:**
- Modify: `codex-rs/core/src/auth.rs`

- [ ] **Step 1: Extend `AuthMode` enum**

The current `AuthMode` has two variants: `ApiKey` and `Chatgpt`. Extend:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthMode {
    /// OpenAI API key (existing).
    ApiKey,
    /// ChatGPT OAuth login (existing).
    Chatgpt,
    /// Generic provider API key (e.g., ANTHROPIC_API_KEY from env).
    ProviderApiKey,
    /// Provider-specific OAuth (e.g., GitHub Copilot, GitLab).
    ProviderOAuth,
    /// No auth needed (local providers).
    NoAuth,
}
```

- [ ] **Step 2: Extend `CodexAuth` enum**

```rust
#[derive(Debug, Clone)]
pub enum CodexAuth {
    ApiKey(ApiKeyAuth),
    Chatgpt(ChatgptAuth),
    ChatgptAuthTokens(ChatgptAuthTokens),
    /// Generic bearer token auth for any provider.
    ProviderBearer { token: String },
    /// API key sent via x-api-key header (Anthropic).
    ProviderApiKeyHeader { key: String },
    /// API key sent as query parameter (Gemini).
    ProviderApiKeyQuery { key: String },
    /// No authentication.
    NoAuth,
}
```

- [ ] **Step 3: Add `resolve_auth_for_provider` function**

Multi-step auth resolution for any provider:

```rust
/// Resolve authentication for a given provider, checking in order:
/// 1. Environment variable(s) from provider config's `env_keys`
/// 2. `experimental_bearer_token` from config.toml
/// 3. Persisted auth in auth.json
/// 4. None (provider may not require auth)
pub fn resolve_auth_for_provider(
    provider: &ProviderConfig,
    auth_json: &AuthDotJson,
) -> Option<CodexAuth> {
    // Check env vars first
    for env_key in &provider.env_keys {
        if let Ok(val) = std::env::var(env_key) {
            if !val.trim().is_empty() {
                return Some(match provider.auth_type {
                    AuthType::ApiKeyHeader => CodexAuth::ProviderApiKeyHeader { key: val },
                    AuthType::QueryParam => CodexAuth::ProviderApiKeyQuery { key: val },
                    _ => CodexAuth::ProviderBearer { token: val },
                });
            }
        }
    }

    // Check persisted auth
    if let Some(provider_auth) = auth_json.get_provider_auth(&provider.id) {
        return Some(provider_auth_to_codex_auth(provider_auth, provider));
    }

    // No auth for local providers
    if provider.auth_type == AuthType::None {
        return Some(CodexAuth::NoAuth);
    }

    None
}
```

- [ ] **Step 4: Commit**

```bash
git add codex-rs/core/src/auth.rs
git commit -m "Extend AuthMode and CodexAuth for multi-provider auth resolution"
```

---

### Task 3: Add GitHub Copilot Device Code OAuth

**Files:**
- Create: `codex-rs/login/src/copilot_auth.rs`
- Modify: `codex-rs/login/src/lib.rs`

- [ ] **Step 1: Implement GitHub device code flow**

GitHub Copilot uses the standard GitHub device code OAuth flow:

```rust
const GITHUB_DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const COPILOT_CLIENT_ID: &str = "Iv1.b507a08c87ecfe98"; // VS Code Copilot client ID
const COPILOT_SCOPES: &str = "read:user";

pub struct CopilotDeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

/// Request a device code for GitHub Copilot authentication.
pub async fn request_copilot_device_code(
    client: &reqwest::Client,
) -> Result<CopilotDeviceCode, CopilotAuthError> { ... }

/// Poll for token completion after user authorizes.
pub async fn complete_copilot_auth(
    client: &reqwest::Client,
    device_code: &CopilotDeviceCode,
) -> Result<ProviderAuth, CopilotAuthError> { ... }

/// Exchange GitHub token for Copilot API token.
/// GET https://api.github.com/copilot_internal/v2/token
async fn get_copilot_token(
    client: &reqwest::Client,
    github_token: &str,
) -> Result<CopilotToken, CopilotAuthError> { ... }
```

- [ ] **Step 2: Add to login crate exports**

In `codex-rs/login/src/lib.rs`, add:
```rust
mod copilot_auth;
pub use copilot_auth::*;
```

- [ ] **Step 3: Commit**

```bash
git add codex-rs/login/src/copilot_auth.rs codex-rs/login/src/lib.rs
git commit -m "Add GitHub Copilot device code OAuth flow"
```

---

### Task 4: Add GitLab OAuth Flow

**Files:**
- Create: `codex-rs/login/src/gitlab_auth.rs`
- Modify: `codex-rs/login/src/lib.rs`

- [ ] **Step 1: Implement GitLab device code flow**

GitLab also supports device code OAuth for headless environments:

```rust
/// GitLab device code OAuth flow.
/// Endpoint: POST https://gitlab.com/oauth/authorize_device
pub async fn request_gitlab_device_code(
    client: &reqwest::Client,
    gitlab_url: &str,  // Allow self-managed GitLab instances
    client_id: &str,
) -> Result<GitLabDeviceCode, GitLabAuthError> { ... }

pub async fn complete_gitlab_auth(
    client: &reqwest::Client,
    gitlab_url: &str,
    device_code: &GitLabDeviceCode,
    client_id: &str,
) -> Result<ProviderAuth, GitLabAuthError> { ... }
```

- [ ] **Step 2: Add to login crate exports**

- [ ] **Step 3: Commit**

```bash
git add codex-rs/login/src/gitlab_auth.rs codex-rs/login/src/lib.rs
git commit -m "Add GitLab OAuth device code flow"
```

---

### Task 5: Add Anthropic API Key Setup Helper

**Files:**
- Create: `codex-rs/login/src/anthropic_auth.rs`
- Modify: `codex-rs/login/src/lib.rs`

- [ ] **Step 1: Implement Anthropic key validation**

Anthropic does not have OAuth -- users enter API keys. But we can validate them:

```rust
/// Validate an Anthropic API key by making a lightweight API call.
pub async fn validate_anthropic_key(
    client: &reqwest::Client,
    api_key: &str,
) -> Result<(), AnthropicAuthError> {
    // POST /v1/messages with minimal payload to check auth
    // Or use a models listing endpoint if available
    let resp = client
        .get("https://api.anthropic.com/v1/models")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await?;

    if resp.status().is_success() || resp.status() == 200 {
        Ok(())
    } else {
        Err(AnthropicAuthError::InvalidKey)
    }
}

/// Store an Anthropic API key in auth.json for persistence.
pub fn save_anthropic_key(
    auth_json: &mut AuthDotJson,
    api_key: String,
) {
    auth_json.set_provider_auth("anthropic", ProviderAuth::ApiKey { key: api_key });
}
```

- [ ] **Step 2: Add to login crate exports**

- [ ] **Step 3: Commit**

```bash
git add codex-rs/login/src/anthropic_auth.rs codex-rs/login/src/lib.rs
git commit -m "Add Anthropic API key validation and storage helper"
```

---

### Task 6: Update TUI Onboarding for Multi-Provider Auth

**Files:**
- Modify: `codex-rs/tui/src/onboarding/auth.rs`
- Modify: `codex-rs/tui/src/onboarding/onboarding_screen.rs`

- [ ] **Step 1: Extend `SignInOption` enum**

Currently the auth screen shows: "Log in (ChatGPT)" and "Use API Key". Extend to show provider-specific options based on the active provider:

```rust
pub(crate) enum SignInOption {
    // Existing
    ChatGptLogin,
    DeviceCodeLogin,
    ApiKeyEntry,

    // New
    AnthropicApiKey,
    GoogleApiKey,
    CopilotOAuth,
    GitLabOAuth,
    GenericApiKey { provider_name: String, env_var: String },
}
```

- [ ] **Step 2: Make auth step provider-aware**

The `AuthModeWidget` should receive the active provider ID and show relevant auth options:
- **OpenAI:** Show existing ChatGPT login + API key options
- **Anthropic:** Show "Enter Anthropic API key (ANTHROPIC_API_KEY)"
- **Google:** Show "Enter Google API key (GOOGLE_API_KEY)"
- **GitHub Copilot:** Show "Log in with GitHub (device code)"
- **GitLab:** Show "Log in with GitLab (device code)"
- **Other providers (Groq, Mistral, etc.):** Show "Enter API key ({env_var})"
- **Ollama/LM Studio:** Skip auth (no key needed)

- [ ] **Step 3: Add API key paste handler**

For providers that use API keys, allow the user to paste the key directly in the TUI:
- Show a text input field
- Validate the key format (basic pattern matching: starts with `sk-ant-` for Anthropic, etc.)
- Store via `save_auth` with the provider ID
- Show success/error feedback

- [ ] **Step 4: Add device code handler for Copilot/GitLab**

For OAuth providers, show the device code and verification URL:
- Request device code from the provider
- Display: "Enter code XXXX-XXXX at https://github.com/login/device"
- Show shimmer animation while polling
- On success, store the token and advance to the next onboarding step

- [ ] **Step 5: Update `OnboardingScreen` step logic**

Modify `codex-rs/tui/src/onboarding/onboarding_screen.rs`:
- If the active provider has auth already (env var set or auth.json entry), skip the Auth step
- If the provider requires no auth (Ollama, LM Studio), skip the Auth step
- Otherwise, show the provider-specific auth step

- [ ] **Step 6: Commit**

```bash
git add codex-rs/tui/src/onboarding/
git commit -m "Update TUI onboarding with multi-provider auth selection"
```

---

### Task 7: Preserve Existing OpenAI Auth

- [ ] **Step 1: Verify backward compatibility**

Ensure the existing OpenAI auth flow is untouched:
- `OPENAI_API_KEY` env var still works
- ChatGPT OAuth login still works
- `auth.json` with legacy format is auto-migrated
- The `orbit_code_login::run_login_server` function is unchanged

- [ ] **Step 2: Run existing auth tests**

```bash
cd codex-rs && cargo test -p orbit-code-core -- auth
cd codex-rs && cargo test -p orbit-code-login
```

- [ ] **Step 3: Full workspace check**

```bash
cd codex-rs && cargo check
```

- [ ] **Step 4: Commit and push**

```bash
git add -A
git commit -m "Stage 5 complete: multi-provider auth system with Copilot/GitLab OAuth and per-provider key storage"
git push origin main
```

---

## Expected Outcomes

After Stage 5:
- `auth.json` stores per-provider credentials keyed by provider ID
- Legacy OpenAI auth.json format auto-migrates to new format
- Auth resolution checks: env var -> config -> auth.json -> none
- GitHub Copilot and GitLab have device-code OAuth flows
- Anthropic has API key validation and persistent storage
- TUI onboarding dynamically shows auth options based on active provider
- Existing OpenAI ApiKey and ChatGPT OAuth flows continue working unchanged
- ~800 lines of new code across core/auth, login, and TUI crates
