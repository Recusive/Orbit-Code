# Fix GPT Mid-Session Switching & Consolidate Anthropic Auth

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix GPT mid-session model switching (broken by Anthropic OAuth token leaking to OpenAI), consolidate scattered Anthropic auth logic into a focused module, and remove debug file writes.

**Architecture:** The root bug is in `auth_cached_for_provider(ProviderName::OpenAI)` which falls through to `self.auth_cached()` returning the Anthropic OAuth token. The fix makes the OpenAI path mirror the Anthropic path — checking env var and v2 storage specifically for OpenAI. The consolidation creates a `core/src/anthropic_auth/` module that owns all Anthropic-specific auth logic (types, refresh, request modifications), reducing the 8-file scatter to a focused module. Debug cleanup replaces `/tmp/` file writes with `tracing::debug!` calls.

**Tech Stack:** Rust, tokio, reqwest, tracing, serde, orbit_code_anthropic crate

---

## File Structure

### Task 1: Fix GPT Mid-Session Switching
- **Modify:** `codex-rs/core/src/auth.rs:1304-1349` — Fix `auth_cached_for_provider` OpenAI path
- **Modify:** `codex-rs/core/src/client.rs:547-552` — Fix `current_client_setup_with_provider` to use correct ProviderName from override
- **Test:** `codex-rs/core/src/auth_tests.rs` — Add tests for `auth_cached_for_provider` with both providers

### Task 2: Clean Up Debug Code
- **Modify:** `codex-rs/core/src/client.rs:1505-1508` — Remove `/tmp/anthropic-request-body.json` write
- **Modify:** `codex-rs/core/src/anthropic_bridge.rs:593-595` — Replace `/tmp/anthropic-sse-error.txt` with tracing
- **Modify:** `codex-rs/anthropic/src/client.rs:127,140-142,165-167,174` — Replace all `/tmp/sse-*.txt` writes with tracing

### Task 3: Fix Lint Violations in auth/anthropic.rs
- **Modify:** `codex-rs/core/src/auth/anthropic.rs:24,32-33` — Remove `#[allow(private_interfaces)]` and `#[allow(dead_code)]`

### Task 4: Consolidate Anthropic Auth Module
- **Create:** `codex-rs/core/src/anthropic_auth/mod.rs` — Module root, re-exports
- **Create:** `codex-rs/core/src/anthropic_auth/types.rs` — Move `AnthropicOAuthAuth`, `AnthropicApiKeyAuth` from `auth/anthropic.rs`
- **Create:** `codex-rs/core/src/anthropic_auth/refresh.rs` — Move `refresh_anthropic_oauth_if_needed`, `force_refresh_anthropic_oauth` from `auth.rs`
- **Create:** `codex-rs/core/src/anthropic_auth/request.rs` — Move `resolve_anthropic_auth`, OAuth request modifications from `client.rs`
- **Deferred:** `codex-rs/core/src/anthropic_auth/model_defaults.rs` — Model defaults stay in `anthropic_bridge.rs` for now (tightly coupled to message building)
- **Modify:** `codex-rs/core/src/auth.rs` — Remove moved refresh functions, update imports
- **Modify:** `codex-rs/core/src/client.rs` — Delegate to `anthropic_auth` module
- **Modify:** `codex-rs/core/src/anthropic_bridge.rs` — Delegate model defaults to new module
- **Modify:** `codex-rs/core/src/lib.rs` — Add `mod anthropic_auth;`
- **Delete:** `codex-rs/core/src/auth/anthropic.rs` — Types moved to new module

---

## Task 1: Fix GPT Mid-Session Switching

### Bug Analysis

The issue is in `auth_cached_for_provider()` at `core/src/auth.rs:1347`:

```rust
ProviderName::OpenAI => self.auth_cached(),
```

When the session started with Anthropic OAuth, `auth_cached()` returns `CodexAuth::AnthropicOAuth`. This leaks the Anthropic token to OpenAI's API endpoint.

There's also a secondary issue in `current_client_setup_with_provider()` at `core/src/client.rs:551`:

```rust
manager.auth_cached_for_provider(ProviderName::OpenAI)
```

This hardcodes `ProviderName::OpenAI` for ALL provider overrides. If we ever support other providers (e.g., OpenRouter), this breaks. It should derive the provider name from the override.

### Files
- Modify: `codex-rs/core/src/auth.rs:1304-1349`
- Modify: `codex-rs/core/src/client.rs:540-563`
- Test: `codex-rs/core/src/auth_tests.rs`

- [ ] **Step 1: Write failing test — OpenAI lookup returns None when only Anthropic cached**

In `codex-rs/core/src/auth_tests.rs`, add:

```rust
#[test]
fn auth_cached_for_provider_openai_does_not_return_anthropic() {
    // When the only auth in storage is Anthropic OAuth, asking for OpenAI should return None.
    let dir = tempdir().unwrap();
    let auth_path = dir.path().join("auth.json");
    let v2 = json!({
        "version": 2,
        "providers": {
            "anthropic": {
                "type": "anthropic_oauth",
                "access_token": "sk-ant-oat01-test",
                "refresh_token": "sk-ant-ort01-test",
                "expires_at": 9999999999_i64
            }
        }
    });
    std::fs::write(&auth_path, serde_json::to_string(&v2).unwrap()).unwrap();

    let manager = AuthManager::new(
        dir.path().to_path_buf(),
        AuthCredentialsStoreMode::File,
        /*enable_orbit_code_api_key_env*/ false,
        /*forced_login_method*/ None,
    )
    .unwrap();

    // OpenAI lookup must NOT return the Anthropic token
    let openai_auth = manager.auth_cached_for_provider(ProviderName::OpenAI);
    assert!(openai_auth.is_none(), "OpenAI lookup should not return Anthropic auth, got: {openai_auth:?}");

    // Anthropic lookup should still work
    let anthropic_auth = manager.auth_cached_for_provider(ProviderName::Anthropic);
    assert!(anthropic_auth.is_some(), "Anthropic lookup should find the OAuth token");
    assert!(matches!(anthropic_auth, Some(CodexAuth::AnthropicOAuth(_))));
}
```

- [ ] **Step 2: Write failing test — OpenAI lookup finds OpenAI from v2 storage**

```rust
#[test]
fn auth_cached_for_provider_openai_finds_openai_in_v2_storage() {
    let dir = tempdir().unwrap();
    let auth_path = dir.path().join("auth.json");
    let v2 = json!({
        "version": 2,
        "providers": {
            "anthropic": {
                "type": "anthropic_oauth",
                "access_token": "sk-ant-oat01-test",
                "refresh_token": "sk-ant-ort01-test",
                "expires_at": 9999999999_i64
            },
            "openai": {
                "type": "openai_api_key",
                "key": "sk-openai-test"
            }
        }
    });
    std::fs::write(&auth_path, serde_json::to_string(&v2).unwrap()).unwrap();

    let manager = AuthManager::new(
        dir.path().to_path_buf(),
        AuthCredentialsStoreMode::File,
        /*enable_orbit_code_api_key_env*/ false,
        /*forced_login_method*/ None,
    )
    .unwrap();

    let openai_auth = manager.auth_cached_for_provider(ProviderName::OpenAI);
    assert!(openai_auth.is_some(), "OpenAI lookup should find OpenAI auth from v2 storage");
}
```

- [ ] **Step 3: Run tests to verify they fail**

Run: `cargo test -p codex-core auth_cached_for_provider`
Expected: FAIL — the first test fails because OpenAI lookup returns AnthropicOAuth; the second test may also fail depending on v2 format.

- [ ] **Step 4: Fix `auth_cached_for_provider` — OpenAI path**

In `codex-rs/core/src/auth.rs`, replace line 1347:

```rust
// BEFORE (buggy):
ProviderName::OpenAI => self.auth_cached(),

// AFTER (fixed):
ProviderName::OpenAI => {
    // Check OPENAI_API_KEY env var
    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        if !key.is_empty() {
            let client = crate::default_client::create_client();
            return Some(CodexAuth::from_api_key_with_client(&key, client));
        }
    }
    // Check persistent v2 storage for OpenAI provider
    if let Ok(Some(v2)) =
        load_auth_dot_json_v2(&self.orbit_code_home, self.auth_credentials_store_mode)
    {
        if v2.provider_auth(ProviderName::OpenAI).is_some() {
            let auth_dot_json = v2.to_v1_openai();
            let client = crate::default_client::create_client();
            if let Ok(auth) = CodexAuth::from_auth_dot_json(
                &self.orbit_code_home,
                auth_dot_json,
                self.auth_credentials_store_mode,
                client,
            ) {
                return Some(auth);
            }
        }
    }
    None
}
```

- [ ] **Step 5: Fix `current_client_setup_with_provider` — derive ProviderName from override**

In `codex-rs/core/src/client.rs`, replace lines 547-555:

```rust
// BEFORE (hardcoded OpenAI):
let auth = match self.state.auth_manager.as_ref() {
    Some(manager) if provider_override.is_some() => {
        use crate::auth::ProviderName;
        manager.auth_cached_for_provider(ProviderName::OpenAI)
    }
    Some(manager) => manager.auth().await,
    None => None,
};

// AFTER (derived from override):
let auth = match self.state.auth_manager.as_ref() {
    Some(manager) if provider_override.is_some() => {
        use crate::auth::ProviderName;
        // Derive provider name from the override's wire_api
        let provider_name = if provider.wire_api == WireApi::AnthropicMessages {
            ProviderName::Anthropic
        } else {
            ProviderName::OpenAI
        };
        manager.auth_cached_for_provider(provider_name)
    }
    Some(manager) => manager.auth().await,
    None => None,
};
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p codex-core auth_cached_for_provider`
Expected: PASS

- [ ] **Step 7: Run clippy on the changed crate**

Run: `just fix -p codex-core`
Expected: No errors

- [ ] **Step 8: Format**

Run: `just fmt`

- [ ] **Step 9: Commit**

```bash
git add codex-rs/core/src/auth.rs codex-rs/core/src/auth_tests.rs codex-rs/core/src/client.rs
git commit -m "fix: GPT mid-session switching no longer leaks Anthropic auth to OpenAI

auth_cached_for_provider(OpenAI) now mirrors the Anthropic path: checks
OPENAI_API_KEY env var and v2 storage for OpenAI-specific auth, returning
None instead of falling through to the cached Anthropic token.

current_client_setup_with_provider derives ProviderName from the override's
wire_api instead of hardcoding OpenAI."
```

---

## Task 2: Clean Up Debug File Writes

Replace all `std::fs::write("/tmp/...")` debug statements with proper `tracing::debug!` calls.

### Files
- Modify: `codex-rs/core/src/client.rs:1505-1508`
- Modify: `codex-rs/core/src/anthropic_bridge.rs:593-595`
- Modify: `codex-rs/anthropic/src/client.rs:127,140-142,165-167,174`

- [ ] **Step 1: Remove `/tmp/anthropic-request-body.json` write from client.rs**

In `codex-rs/core/src/client.rs`, remove lines 1505-1508:

```rust
// DELETE these lines:
// Dump the full request body for debugging
if let Ok(json) = serde_json::to_string_pretty(&request) {
    let _ = std::fs::write("/tmp/anthropic-request-body.json", &json);
}
```

The `tracing::info!` at lines 1497-1503 already logs the key request details (model, thinking, beta_headers, max_tokens). No replacement needed.

- [ ] **Step 2: Replace `/tmp/anthropic-sse-error.txt` in anthropic_bridge.rs**

In `codex-rs/core/src/anthropic_bridge.rs`, replace lines 593-595:

```rust
// BEFORE:
let _ = std::fs::write("/tmp/anthropic-sse-error.txt", format!(
    "error_type={error_type} message={message}"
));

// AFTER:
tracing::debug!(
    error_type = %error_type,
    message = %message,
    "Anthropic SSE error event"
);
```

- [ ] **Step 3: Replace all `/tmp/sse-*.txt` writes in anthropic/src/client.rs**

In `codex-rs/anthropic/src/client.rs`, make these replacements:

**Line 127** — transport error:
```rust
// BEFORE:
let _ = std::fs::write("/tmp/sse-transport-error.txt", format!("{e:?}"));

// AFTER:
tracing::debug!(error = ?e, "Anthropic SSE transport error");
```

**Lines 140-142** — raw SSE error event:
```rust
// BEFORE:
let _ = std::fs::write("/tmp/sse-raw-error.txt", format!(
    "event_name={} data={}", event.event, event.data
));

// AFTER:
tracing::debug!(
    event_name = %event.event,
    data = %event.data,
    "Anthropic SSE raw error event"
);
```

**Lines 165-167** — eventsource error:
```rust
// BEFORE:
let _ = std::fs::write("/tmp/sse-eventsource-error.txt", format!(
    "eventsource error: {err}"
));

// AFTER:
tracing::debug!(error = %err, "Anthropic SSE eventsource error");
```

**Line 174** — stream closed:
```rust
// BEFORE:
let _ = std::fs::write("/tmp/sse-stream-closed.txt", "stream closed before message_stop");

// AFTER:
tracing::debug!("Anthropic SSE stream closed before message_stop");
```

- [ ] **Step 4: Add tracing dependency to anthropic crate if missing**

Check `codex-rs/anthropic/Cargo.toml` for `tracing` dependency. If missing, add:
```toml
tracing = { workspace = true }
```

- [ ] **Step 5: Build to verify no compile errors**

Run: `cargo build -p codex-core -p codex-anthropic`
Expected: Compiles successfully with no errors

- [ ] **Step 6: Run clippy**

Run: `just fix -p codex-core && just fix -p codex-anthropic`

- [ ] **Step 7: Format**

Run: `just fmt`

- [ ] **Step 8: Commit**

```bash
git add codex-rs/core/src/client.rs codex-rs/core/src/anthropic_bridge.rs codex-rs/anthropic/src/client.rs codex-rs/anthropic/Cargo.toml
git commit -m "chore: replace /tmp debug file writes with tracing::debug!

Remove all std::fs::write(\"/tmp/...\") debug statements from Anthropic
client code. Transport and SSE errors are now logged via tracing::debug!
for proper observability without filesystem side effects."
```

---

## Task 3: Fix Lint Violations in auth/anthropic.rs

The existing `auth/anthropic.rs` has two `#[allow(...)]` annotations that violate the zero-tolerance lint policy.

### Files
- Modify: `codex-rs/core/src/auth/anthropic.rs:24,32-33`

- [ ] **Step 1: Remove `#[allow(private_interfaces)]` from AnthropicOAuthAuth**

The `#[allow(private_interfaces)]` at line 24 exists because `AuthStorageBackend` is `pub(crate)`. Since `AnthropicOAuthAuth` is used within the same crate, make the struct fields that reference `AuthStorageBackend` use `pub(crate)` visibility consistently. The fix: make `AnthropicOAuthAuth` itself `pub(crate)` visibility... but it's used in `CodexAuth::AnthropicOAuth(AnthropicOAuthAuth)` which is public.

The real fix: `AuthStorageBackend` trait is already `pub(crate)`. The struct field `storage` is `pub(crate)`. The warning fires because the struct is `pub` but exposes a `pub(crate)` type in its definition. Since the struct is only constructed within the crate, we can suppress this differently:

Actually, the correct fix is to remove the `#[allow(private_interfaces)]` and instead make the struct not leak the private type. The `storage` field is already `pub(crate)`, which means external crate users can't access it. The issue is that Rust still considers it a private interface leak. The fix: wrap the storage in a type-erased `Arc<dyn Any>` or use a concrete public wrapper. But the simplest fix: remove the `storage` field entirely since it's `#[allow(dead_code)]` — it's unused!

```rust
// BEFORE:
#[allow(private_interfaces)]
#[derive(Debug, Clone)]
pub struct AnthropicOAuthAuth {
    pub(crate) access_token: String,
    pub(crate) refresh_token: String,
    pub(crate) expires_at: i64,
    #[allow(dead_code)]
    pub(crate) storage: Arc<dyn AuthStorageBackend>,
}

// AFTER:
#[derive(Debug, Clone)]
pub struct AnthropicOAuthAuth {
    pub(crate) access_token: String,
    pub(crate) refresh_token: String,
    pub(crate) expires_at: i64,
}
```

Also remove `use std::sync::Arc;` and `use super::storage::AuthStorageBackend;` if they become unused.

- [ ] **Step 2: Update `codex_auth_from_provider_auth` to stop passing storage**

In `codex-rs/core/src/auth.rs`, the function at line ~759 passes `storage` to `AnthropicOAuthAuth`. Remove the `storage` field:

```rust
// BEFORE:
Some(CodexAuth::AnthropicOAuth(
    crate::auth::anthropic::AnthropicOAuthAuth {
        access_token: access_token.clone(),
        refresh_token: refresh_token.clone(),
        expires_at: *expires_at,
        storage,
    },
))

// AFTER:
Some(CodexAuth::AnthropicOAuth(
    crate::auth::anthropic::AnthropicOAuthAuth {
        access_token: access_token.clone(),
        refresh_token: refresh_token.clone(),
        expires_at: *expires_at,
    },
))
```

Also remove the `let storage = create_auth_storage(...)` line at ~756-757 since it's no longer needed.

- [ ] **Step 3: Check for other construction sites of AnthropicOAuthAuth**

Run: `grep -rn "AnthropicOAuthAuth {" codex-rs/` and update any that pass `storage`.

- [ ] **Step 4: Build and test**

Run: `cargo build -p codex-core && cargo test -p codex-core`
Expected: Compiles with no lint violations, tests pass

- [ ] **Step 5: Format**

Run: `just fmt`

- [ ] **Step 6: Commit**

```bash
git add codex-rs/core/src/auth/anthropic.rs codex-rs/core/src/auth.rs
git commit -m "fix: remove lint suppressions from AnthropicOAuthAuth

Remove unused storage field and its #[allow(dead_code)]. This also
eliminates the #[allow(private_interfaces)] since the struct no longer
references the crate-private AuthStorageBackend trait."
```

---

## Task 4: Consolidate Anthropic Auth Module

Move Anthropic-specific auth logic from 6+ scattered locations into a focused `core/src/anthropic_auth/` module.

### Current scatter:
| Location | Anthropic logic |
|----------|----------------|
| `core/src/auth/anthropic.rs` | `AnthropicOAuthAuth`, `AnthropicApiKeyAuth` types |
| `core/src/auth.rs` | `refresh_anthropic_oauth_if_needed()`, `force_refresh_anthropic_oauth()`, `UnauthorizedRecovery::AnthropicOAuth` |
| `core/src/client.rs` | `resolve_anthropic_auth()`, OAuth system prompt prefix, tool prefixing call, proactive refresh call |
| `core/src/anthropic_bridge.rs` | `anthropic_model_defaults()`, `uses_adaptive_thinking()`, `prefix_tool_names_for_oauth()`, `strip_oauth_tool_prefix()`, `merge_anthropic_beta_headers()`, `is_known_anthropic_model()` |
| `anthropic/src/token_refresh.rs` | `refresh_anthropic_token()` HTTP call |

### Target structure:
```
core/src/anthropic_auth/
├── mod.rs              — Re-exports, module docs
├── types.rs            — AnthropicOAuthAuth, AnthropicApiKeyAuth, constants
├── refresh.rs          — Token refresh (proactive + forced), delegates to anthropic crate
├── request.rs          — resolve_anthropic_auth, apply_oauth_modifications (system prompt, tool prefix)
└── model_defaults.rs   — anthropic_model_defaults, uses_adaptive_thinking, beta headers
```

### Files
- Create: `codex-rs/core/src/anthropic_auth/mod.rs`
- Create: `codex-rs/core/src/anthropic_auth/types.rs`
- Create: `codex-rs/core/src/anthropic_auth/refresh.rs`
- Create: `codex-rs/core/src/anthropic_auth/request.rs`
- Create: `codex-rs/core/src/anthropic_auth/model_defaults.rs`
- Modify: `codex-rs/core/src/lib.rs`
- Modify: `codex-rs/core/src/auth.rs`
- Modify: `codex-rs/core/src/client.rs`
- Modify: `codex-rs/core/src/anthropic_bridge.rs`
- Delete: `codex-rs/core/src/auth/anthropic.rs`

**Note:** This task is a pure refactor. No behavior changes. Every function must produce identical results before and after.

- [ ] **Step 1: Create `anthropic_auth/types.rs`**

Move `AnthropicOAuthAuth`, `AnthropicApiKeyAuth`, and `ANTHROPIC_TOKEN_REFRESH_BUFFER_SECONDS` from `auth/anthropic.rs`:

```rust
//! Anthropic-specific authentication types.

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
}

impl AnthropicOAuthAuth {
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn refresh_token(&self) -> &str {
        &self.refresh_token
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

- [ ] **Step 2: Create `anthropic_auth/refresh.rs`**

Move `refresh_anthropic_oauth_if_needed` and `force_refresh_anthropic_oauth` from `auth.rs`. These functions stay on `AuthManager` but the implementation logic moves here as free functions that `AuthManager` delegates to:

```rust
//! Anthropic OAuth token refresh logic.
//!
//! Provides proactive (before-expiry) and forced (on-401) refresh of
//! Anthropic OAuth access tokens, persisting refreshed tokens to storage.

use crate::auth::AuthManager;
use crate::auth::CodexAuth;
use crate::auth::storage::AuthCredentialsStoreMode;
use crate::auth::storage::ProviderAuth;
use crate::auth::storage::ProviderName;
use crate::auth::storage::create_auth_storage;
use crate::token_data::RefreshTokenError;
use crate::token_data::RefreshTokenFailedError;
use crate::token_data::RefreshTokenFailedReason;
use super::types::ANTHROPIC_TOKEN_REFRESH_BUFFER_SECONDS;

/// Proactively refresh if the Anthropic OAuth token is expiring soon.
/// Best-effort: failures are logged but don't block requests.
pub async fn refresh_if_needed(manager: &AuthManager) {
    let auth = match manager.auth_cached_for_provider(ProviderName::Anthropic) {
        Some(CodexAuth::AnthropicOAuth(ref oauth)) => oauth.clone(),
        _ => return,
    };

    if !auth.is_expiring_within(ANTHROPIC_TOKEN_REFRESH_BUFFER_SECONDS) {
        return;
    }

    tracing::info!("Anthropic OAuth token expiring soon, refreshing proactively");
    let client = crate::default_client::build_reqwest_client();
    match orbit_code_anthropic::refresh_anthropic_token(&client, auth.refresh_token()).await {
        Ok(tokens) => {
            persist_refreshed_tokens(manager, tokens);
        }
        Err(e) => {
            tracing::warn!("Proactive Anthropic OAuth refresh failed: {e}");
        }
    }
}

/// Force-refresh an Anthropic OAuth token (e.g., after 401).
/// Returns a typed error unlike the proactive variant.
pub async fn force_refresh(
    manager: &AuthManager,
) -> std::result::Result<(), RefreshTokenError> {
    let auth = manager.auth_cached_for_provider(ProviderName::Anthropic);
    let Some(CodexAuth::AnthropicOAuth(ref oauth)) = auth else {
        return Err(RefreshTokenError::Permanent(RefreshTokenFailedError::new(
            RefreshTokenFailedReason::Other,
            "not AnthropicOAuth",
        )));
    };
    let client = crate::default_client::build_reqwest_client();
    let tokens = orbit_code_anthropic::refresh_anthropic_token(&client, oauth.refresh_token())
        .await
        .map_err(|e| RefreshTokenError::Transient(std::io::Error::other(e.to_string())))?;

    persist_refreshed_tokens(manager, tokens);
    Ok(())
}

/// Persist refreshed Anthropic OAuth tokens to v2 storage and reload the cache.
fn persist_refreshed_tokens(
    manager: &AuthManager,
    tokens: orbit_code_anthropic::RefreshedTokens,
) {
    let now = chrono::Utc::now().timestamp();
    let expires_at = now.saturating_add(i64::try_from(tokens.expires_in).unwrap_or(3600));
    let storage = create_auth_storage(
        manager.orbit_code_home().to_path_buf(),
        manager.auth_credentials_store_mode(),
    );
    match storage.load() {
        Ok(Some(mut v2)) => {
            v2.set_provider_auth(
                ProviderName::Anthropic,
                ProviderAuth::AnthropicOAuth {
                    access_token: tokens.access_token,
                    refresh_token: tokens.refresh_token,
                    expires_at,
                },
            );
            match storage.save(&v2) {
                Ok(()) => {
                    manager.reload();
                }
                Err(e) => tracing::warn!(
                    "Anthropic refresh succeeded but persist failed: {e}"
                ),
            }
        }
        Ok(None) => {
            tracing::warn!(
                "Anthropic refresh succeeded but no auth storage found; keeping cached token"
            );
        }
        Err(e) => {
            tracing::warn!(
                "Anthropic refresh succeeded but storage unreadable: {e}; keeping cached token"
            );
        }
    }
}
```

**Note:** This requires adding `pub fn orbit_code_home(&self) -> &Path` and `pub fn auth_credentials_store_mode(&self) -> AuthCredentialsStoreMode` accessor methods on `AuthManager` if they don't exist.

- [ ] **Step 3: Create `anthropic_auth/request.rs`**

Move `resolve_anthropic_auth` and OAuth request modification logic from `client.rs`:

```rust
//! Anthropic auth resolution and OAuth request modifications.
//!
//! Resolves the correct AnthropicAuth (Bearer or API key) from the auth manager,
//! provider config, and environment. Applies OAuth-specific request transformations
//! (system prompt prefix, tool name prefixing).

use crate::auth::AuthManager;
use crate::auth::CodexAuth;
use crate::auth::storage::ProviderName;
use crate::error::CodexErr;
use crate::error::EnvVarError;
use crate::error::Result;
use crate::model_provider_info::ModelProviderInfo;
use http::HeaderMap as ApiHeaderMap;
use orbit_code_anthropic::AnthropicAuth;
use orbit_code_anthropic::MessagesRequest;
use orbit_code_anthropic::SystemBlock;

const OAUTH_TOOL_PREFIX: &str = "mcp_";

/// Resolve Anthropic authentication from AuthManager, falling back to
/// provider config headers and env vars.
pub(crate) fn resolve_auth(
    auth_manager: Option<&AuthManager>,
    provider: &ModelProviderInfo,
    extra_headers: &ApiHeaderMap,
) -> Result<AnthropicAuth> {
    // 1. Check AuthManager for stored Anthropic credentials
    if let Some(manager) = auth_manager {
        let found = manager.auth_cached_for_provider(ProviderName::Anthropic);
        tracing::info!(
            found = found.is_some(),
            auth_mode = ?found.as_ref().map(|a| a.auth_mode()),
            "resolve_anthropic_auth: checking AuthManager"
        );
        if let Some(auth) = found {
            match auth {
                CodexAuth::AnthropicOAuth(ref oauth) => {
                    return Ok(AnthropicAuth::BearerToken(oauth.access_token().to_string()));
                }
                CodexAuth::AnthropicApiKey(ref api_key_auth) => {
                    return Ok(AnthropicAuth::ApiKey(api_key_auth.api_key().to_string()));
                }
                _ => {
                    tracing::warn!(auth_mode = ?auth.auth_mode(), "resolve_anthropic_auth: unexpected variant");
                }
            }
        }
    } else {
        tracing::warn!("resolve_anthropic_auth: no AuthManager available");
    }

    // 2. Fall back to provider config headers or env var
    if let Some(api_key) = extra_headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .or(provider.api_key()?)
    {
        return Ok(AnthropicAuth::ApiKey(api_key));
    }

    // 3. Check experimental bearer token
    if let Some(bearer) = &provider.experimental_bearer_token {
        return Ok(AnthropicAuth::BearerToken(bearer.clone()));
    }

    Err(CodexErr::EnvVar(EnvVarError {
        var: "ANTHROPIC_API_KEY".to_string(),
        instructions: provider.env_key_instructions.clone(),
    }))
}

/// Apply OAuth-specific modifications to an Anthropic request:
/// - Prefix tool names with `mcp_`
/// - Prepend the required "You are Claude Code..." system block
pub(crate) fn apply_oauth_modifications(request: &mut MessagesRequest) {
    // Prefix tool names (definitions + history)
    prefix_tool_names(request);

    // Prepend required system prompt block
    let prefix_block = SystemBlock {
        r#type: "text".to_string(),
        text: "You are Claude Code, Anthropic's official CLI for Claude.".to_string(),
        cache_control: None,
    };
    if let Some(system) = &mut request.system {
        system.insert(0, prefix_block);
    } else {
        request.system = Some(vec![prefix_block]);
    }
}

/// Prefix tool names with `mcp_` for OAuth mode (Anthropic requirement).
fn prefix_tool_names(request: &mut MessagesRequest) {
    if let Some(tools) = &mut request.tools {
        for tool in tools {
            if !tool.name.starts_with(OAUTH_TOOL_PREFIX) {
                tool.name = format!("{OAUTH_TOOL_PREFIX}{}", tool.name);
            }
        }
    }
    for message in &mut request.messages {
        if let orbit_code_anthropic::Content::Blocks(blocks) = &mut message.content {
            for block in blocks {
                if let orbit_code_anthropic::ContentBlock::ToolUse { name, .. } = block {
                    if !name.starts_with(OAUTH_TOOL_PREFIX) {
                        *name = format!("{OAUTH_TOOL_PREFIX}{name}");
                    }
                }
            }
        }
    }
}

/// Strip `mcp_` prefix from tool names in responses (Anthropic requirement).
pub(crate) fn strip_oauth_tool_prefix(name: &str) -> String {
    name.strip_prefix(OAUTH_TOOL_PREFIX)
        .unwrap_or(name)
        .to_string()
}
```

- [ ] **Step 4: Create `anthropic_auth/model_defaults.rs`**

Move `anthropic_model_defaults`, `uses_adaptive_thinking`, `requires_1m_context`, `supports_effort_parameter`, and beta header merge logic from `anthropic_bridge.rs`. These functions stay in `anthropic_bridge.rs` for now but are re-exported from the new module as a stepping stone.

**Decision:** Since `anthropic_bridge.rs` already has significant non-auth logic (message building, event mapping, stream bridging), and model defaults are tightly coupled to the bridge's request building, keep model defaults in `anthropic_bridge.rs` for this phase. Only move auth-specific logic.

Skip this file for now — the types, refresh, and request modules cover the critical auth consolidation.

- [ ] **Step 5: Create `anthropic_auth/mod.rs`**

```rust
//! Consolidated Anthropic authentication module.
//!
//! Owns all Anthropic-specific auth logic:
//! - OAuth and API key auth types
//! - Token refresh (proactive and forced)
//! - Auth resolution for Anthropic API requests
//! - OAuth-specific request modifications (tool prefixing, system prompt)

mod refresh;
mod request;
mod types;

pub use types::AnthropicApiKeyAuth;
pub use types::AnthropicOAuthAuth;
pub use types::ANTHROPIC_TOKEN_REFRESH_BUFFER_SECONDS;

pub(crate) use refresh::force_refresh;
pub(crate) use refresh::refresh_if_needed;
pub(crate) use request::apply_oauth_modifications;
pub(crate) use request::resolve_auth;
pub(crate) use request::strip_oauth_tool_prefix;
```

- [ ] **Step 6: Add module to `lib.rs`**

In `codex-rs/core/src/lib.rs`, add:
```rust
mod anthropic_auth;
```

Also update the re-export of `AnthropicOAuthAuth` and `AnthropicApiKeyAuth` if they're currently re-exported from `auth`.

- [ ] **Step 7: Delete `auth/anthropic.rs` and update `auth.rs` imports**

In `codex-rs/core/src/auth.rs`:
- Remove `mod anthropic;` declaration
- Replace `use crate::auth::anthropic::*` with `use crate::anthropic_auth::*`
- Replace the inline `refresh_anthropic_oauth_if_needed` and `force_refresh_anthropic_oauth` methods on `AuthManager` to delegate:

```rust
pub async fn refresh_anthropic_oauth_if_needed(&self) {
    crate::anthropic_auth::refresh_if_needed(self).await;
}

pub async fn force_refresh_anthropic_oauth(
    &self,
) -> std::result::Result<(), RefreshTokenError> {
    crate::anthropic_auth::force_refresh(self).await
}
```

Add accessor methods needed by the refresh module:
```rust
pub fn orbit_code_home(&self) -> &Path {
    &self.orbit_code_home
}

pub fn auth_credentials_store_mode(&self) -> AuthCredentialsStoreMode {
    self.auth_credentials_store_mode
}
```

- [ ] **Step 8: Update `client.rs` to use consolidated module**

In `codex-rs/core/src/client.rs`:
- Replace `self.resolve_anthropic_auth(provider, &extra_headers)` with `crate::anthropic_auth::resolve_auth(self.client.state.auth_manager.as_deref(), provider, &extra_headers)`
- Replace the inline OAuth modification block with `crate::anthropic_auth::apply_oauth_modifications(&mut request)`
- Remove the `resolve_anthropic_auth` method from `ModelClientSession`
- Remove the import of `prefix_tool_names_for_oauth` from `anthropic_bridge`

- [ ] **Step 9: Update `anthropic_bridge.rs` to remove moved code**

In `codex-rs/core/src/anthropic_bridge.rs`:
- Remove `prefix_tool_names_for_oauth` function (moved to `anthropic_auth/request.rs`)
- Remove `strip_oauth_tool_prefix` function (moved to `anthropic_auth/request.rs`)
- Update the stream mapping function to import `strip_oauth_tool_prefix` from `crate::anthropic_auth`
- Remove the `OAUTH_TOOL_PREFIX` constant

- [ ] **Step 10: Build full workspace**

Run: `cargo build -p codex-core`
Expected: Compiles with no errors

- [ ] **Step 11: Run tests**

Run: `cargo test -p codex-core`
Expected: All tests pass (pure refactor, no behavior changes)

- [ ] **Step 12: Run clippy**

Run: `just fix -p codex-core`

- [ ] **Step 13: Format**

Run: `just fmt`

- [ ] **Step 14: Commit**

```bash
git add codex-rs/core/src/anthropic_auth/ codex-rs/core/src/lib.rs codex-rs/core/src/auth.rs codex-rs/core/src/auth/anthropic.rs codex-rs/core/src/client.rs codex-rs/core/src/anthropic_bridge.rs
git commit -m "refactor: consolidate Anthropic auth into dedicated module

Create core/src/anthropic_auth/ module owning:
- types.rs: AnthropicOAuthAuth, AnthropicApiKeyAuth
- refresh.rs: proactive + forced OAuth token refresh
- request.rs: auth resolution, OAuth request modifications

auth.rs delegates refresh to the new module. client.rs delegates auth
resolution and OAuth modifications. anthropic_bridge.rs no longer owns
tool prefixing logic.

Reduces Anthropic auth scatter from 6+ files to a focused module."
```

---

## Verification Checklist

After all tasks are complete:

- [ ] `cargo build -p codex-core -p codex-anthropic` — compiles clean
- [ ] `cargo test -p codex-core` — all tests pass
- [ ] `just fix -p codex-core && just fix -p codex-anthropic` — no clippy warnings
- [ ] `just fmt` — no formatting changes
- [ ] `grep -rn 'std::fs::write("/tmp' codex-rs/core/ codex-rs/anthropic/` — no debug writes remain
- [ ] `grep -rn '#\[allow(' codex-rs/core/src/auth/` — no lint suppressions in auth module
- [ ] Manual test: start on Claude → switch to GPT → switch back to Claude → verify `/status` for each
