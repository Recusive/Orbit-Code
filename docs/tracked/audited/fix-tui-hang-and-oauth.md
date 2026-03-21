# Fix TUI Hang on Anthropic Provider + Add OAuth to Claude Sign-In (v3)

> v3: Fixes crate cycle (Critical #1), async refresh design (Critical #2), and `UnauthorizedRecovery` Anthropic path (Critical #3) from v2 audit.

## Context

After Stage 3b, the TUI hangs when sending messages via the Anthropic provider. Esc/Ctrl+C become unresponsive. Additionally, the "Sign in with Claude" TUI option only offers API key entry — no OAuth code-paste flow.

## Root Causes

1. **No connect_timeout on reqwest client** — TCP connect can hang indefinitely.
2. **Spawned SSE reader task survives cancellation** — `AnthropicStream` lacks `JoinHandle` abort-on-drop.
3. **Anthropic login helpers use raw `reqwest::Client::new()`** — bypasses CA-aware builder, no timeouts.
4. **No Anthropic OAuth token refresh/recovery** — runtime cannot refresh expired tokens or recover from 401.

---

## Step 1: Fix HTTP clients

### 1a: Add connect_timeout to shared reqwest client

**File:** `core/src/default_client.rs`

Add `.connect_timeout(Duration::from_secs(10))` to `try_build_reqwest_client()` builder chain at line 207.

### 1b: Fix Anthropic login helpers to use CA-aware client with timeout

**File:** `login/src/anthropic.rs`

Add a new error variant for client build failures:
```rust
#[error("Failed to build HTTP client: {0}")]
ClientBuild(String),
```

Create a shared helper using the CA-aware builder:
```rust
fn build_anthropic_login_client() -> Result<reqwest::Client, AnthropicLoginError> {
    orbit_code_client::build_reqwest_client_with_custom_ca(
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
    ).map_err(|e| AnthropicLoginError::ClientBuild(e.to_string()))
}
```

Replace all `Client::new()` calls in `anthropic_exchange_code()`, `anthropic_refresh_token()`, and `anthropic_create_api_key()`.

---

## Step 2: Abort SSE reader task on stream drop

**File:** `anthropic/src/client.rs`

```rust
pub struct AnthropicStream {
    rx: mpsc::Receiver<Result<AnthropicEvent>>,
    _reader_task: tokio::task::JoinHandle<()>,
}

impl Drop for AnthropicStream {
    fn drop(&mut self) {
        self._reader_task.abort();
    }
}
```

Update `stream()` to capture the `JoinHandle` from `tokio::spawn` and store it.

---

## Step 3: Anthropic OAuth token refresh/recovery (v3 — fixes crate cycle and control flow)

### Design decision: refresh logic lives in `orbit-code-anthropic`, not `orbit-code-login`

**Why:** `core` already depends on `orbit-code-anthropic`, but NOT on `orbit-code-login` (which depends on `core` — cycle). The Anthropic token refresh HTTP call is a simple POST to the token endpoint — it belongs in the same crate as the Anthropic API client.

### 3a: Add token refresh function to `orbit-code-anthropic`

**New file:** `anthropic/src/token_refresh.rs`

```rust
//! Anthropic OAuth token refresh — minimal HTTP client for token endpoint.

use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;

const ANTHROPIC_TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";
const ANTHROPIC_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";

#[derive(Debug, Deserialize)]
pub struct RefreshedTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

/// Refresh an Anthropic OAuth access token using a refresh token.
/// Uses JSON body (not form-encoded), matching the Anthropic token endpoint spec.
pub async fn refresh_anthropic_token(
    client: &Client,
    refresh_token: &str,
) -> Result<RefreshedTokens, crate::AnthropicError> {
    #[derive(Serialize)]
    struct RefreshRequest<'a> {
        grant_type: &'static str,
        refresh_token: &'a str,
        client_id: &'static str,
    }

    let resp = client
        .post(ANTHROPIC_TOKEN_URL)
        .json(&RefreshRequest {
            grant_type: "refresh_token",
            refresh_token,
            client_id: ANTHROPIC_CLIENT_ID,
        })
        .send()
        .await
        .map_err(|e| crate::AnthropicError::Transport(
            orbit_code_client::TransportError::Network(e.to_string())
        ))?;

    if !resp.status().is_success() {
        let status = resp.status().as_u16();
        let body = resp.text().await.ok();
        return Err(crate::AnthropicError::Api(Box::new(crate::AnthropicApiError {
            status,
            error_type: "token_refresh_failed".to_string(),
            message: body.unwrap_or_else(|| "refresh failed".to_string()),
        })));
    }

    resp.json::<RefreshedTokens>()
        .await
        .map_err(|e| crate::AnthropicError::Transport(
            orbit_code_client::TransportError::Network(format!("failed to decode refresh response: {e}"))
        ))
}
```

Export from `anthropic/src/lib.rs`: `pub use token_refresh::refresh_anthropic_token;` and `pub use token_refresh::RefreshedTokens;`

The function takes a `&Client` parameter so callers can pass their CA-aware client — no raw `Client::new()`.

### 3b: Add `refresh_anthropic_oauth_if_needed()` to `AuthManager`

**File:** `core/src/auth.rs`

`AuthManager` already owns `orbit_code_home`, `auth_credentials_store_mode`, and has access to `create_auth_storage` (which is `pub(super)` in the same `auth` module). Add a public async method:

```rust
/// Proactively refresh Anthropic OAuth if the token is expiring soon.
/// Best-effort: failures are logged but don't prevent the request from proceeding
/// (the stale token might still work, and 401 recovery handles the rest).
pub async fn refresh_anthropic_oauth_if_needed(&self) {
    let auth = match self.auth_cached_for_provider(ProviderName::Anthropic) {
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
            let now = chrono::Utc::now().timestamp();
            let expires_at = now.saturating_add(i64::try_from(tokens.expires_in).unwrap_or(3600));
            // Persist refreshed tokens. ONLY reload cache after a successful save.
            // If storage fails, keep the current cached token (it might still work)
            // and let 401 recovery handle it later.
            let storage = create_auth_storage(
                self.orbit_code_home.clone(),
                self.auth_credentials_store_mode,
            );
            match storage.load() {
                Ok(Some(mut v2)) => {
                    v2.set_provider_auth(ProviderName::Anthropic, ProviderAuth::AnthropicOAuth {
                        access_token: tokens.access_token,
                        refresh_token: tokens.refresh_token,
                        expires_at,
                    });
                    match storage.save(&v2) {
                        Ok(()) => self.reload(), // Only reload on successful persist
                        Err(e) => tracing::warn!("Anthropic refresh succeeded but persist failed: {e}"),
                    }
                }
                Ok(None) => {
                    tracing::warn!("Anthropic refresh succeeded but no auth storage found; keeping cached token");
                }
                Err(e) => {
                    tracing::warn!("Anthropic refresh succeeded but storage unreadable: {e}; keeping cached token");
                }
            }
        }
        Err(e) => {
            tracing::warn!("Proactive Anthropic OAuth refresh failed: {e}");
        }
    }
}
```

### 3b-call: Call from `stream_anthropic_messages()`

**File:** `core/src/client.rs` — inside `stream_anthropic_messages()` (async fn)

The key insight: `resolve_anthropic_auth()` is synchronous, but `stream_anthropic_messages()` is async. Call the `AuthManager` method before the sync resolver:

```rust
async fn stream_anthropic_messages(&self, ...) -> Result<ResponseStream> {
    // ... existing model validation and request building ...

    // Proactive Anthropic OAuth refresh (async, before sync auth resolution)
    if let Some(manager) = self.client.state.auth_manager.as_ref() {
        manager.refresh_anthropic_oauth_if_needed().await;
    }

    // Existing: sync auth resolution (reads freshly-reloaded cache)
    let anthropic_auth = self.resolve_anthropic_auth(provider, &extra_headers)?;
    // ... rest unchanged ...
}
```

This design:
- `client.rs` calls ONE method on `AuthManager` — no direct storage or config access needed
- `AuthManager` owns all persist/reload logic (it already has `orbit_code_home` and storage access)
- Best-effort: failures are logged, request proceeds with existing token
- After refresh, `self.reload()` makes the new token visible to subsequent sync `resolve_anthropic_auth()`

### 3c: Add Anthropic 401 recovery to `UnauthorizedRecovery`

**File:** `core/src/auth.rs`

The current `UnauthorizedRecovery` is ChatGPT-shaped:
- `has_next()` checks `is_chatgpt_auth()`
- `expected_account_id` from `get_account_id()`
- `reload_if_account_id_matches(None)` immediately skips for Anthropic (no account id)

**Fix: Add a separate Anthropic recovery mode** that skips the account-id guard entirely:

```rust
enum UnauthorizedRecoveryMode {
    Managed,           // existing: ChatGPT with account-id guard
    External,          // existing: ChatGPT external tokens
    AnthropicOAuth,    // NEW: skip account-id guard, go straight to refresh
}
```

In `UnauthorizedRecovery::new()`:
```rust
let mode = if cached_auth.as_ref().is_some_and(CodexAuth::is_external_chatgpt_tokens) {
    UnauthorizedRecoveryMode::External
} else if matches!(cached_auth.as_ref(), Some(CodexAuth::AnthropicOAuth(_))) {
    UnauthorizedRecoveryMode::AnthropicOAuth
} else {
    UnauthorizedRecoveryMode::Managed
};
```

Update `has_next()`:
```rust
pub fn has_next(&self) -> bool {
    let auth = self.manager.auth_cached();
    let is_recoverable = auth.as_ref().is_some_and(|a| {
        a.is_chatgpt_auth() || matches!(a, CodexAuth::AnthropicOAuth(_))
    });
    if !is_recoverable { return false; }
    if self.mode == UnauthorizedRecoveryMode::External && !self.manager.has_external_auth_refresher() {
        return false;
    }
    !matches!(self.step, UnauthorizedRecoveryStep::Done)
}
```

Update `next()` for `AnthropicOAuth` mode — two steps, no account-id guard:
```rust
UnauthorizedRecoveryMode::AnthropicOAuth => match self.step {
    UnauthorizedRecoveryStep::Reload => {
        // Skip account-id guard — just reload from storage
        // (another process may have refreshed the token)
        let old_auth = self.manager.auth_cached();
        self.manager.reload();
        let new_auth = self.manager.auth_cached();
        let changed = old_auth != new_auth;
        self.step = UnauthorizedRecoveryStep::RefreshToken;
        Ok(UnauthorizedRecoveryStepResult { auth_state_changed: Some(changed) })
    }
    UnauthorizedRecoveryStep::RefreshToken => {
        // Refresh using orbit-code-anthropic (no crate cycle)
        // Same approach as maybe_refresh_anthropic_oauth() — call refresh_anthropic_token,
        // persist, reload
        self.refresh_anthropic_oauth_token().await?;
        self.step = UnauthorizedRecoveryStep::Done;
        Ok(UnauthorizedRecoveryStepResult { auth_state_changed: Some(true) })
    }
    _ => { self.step = UnauthorizedRecoveryStep::Done; /* no more steps */ }
}
```

The `RefreshToken` step for `AnthropicOAuth` mode delegates to `AuthManager`:
```rust
UnauthorizedRecoveryStep::RefreshToken => {
    // Force refresh — we got a 401, the token is definitely bad.
    // force_refresh_anthropic_oauth() returns typed RefreshTokenError.
    // It only reloads cache after successful persist (no regression on storage failure).
    self.manager.force_refresh_anthropic_oauth().await?;
    self.step = UnauthorizedRecoveryStep::Done;
    Ok(UnauthorizedRecoveryStepResult { auth_state_changed: Some(true) })
}
```

Add `force_refresh_anthropic_oauth()` to `AuthManager` — same persist-then-reload logic, but without the expiry check (always refreshes) and with a typed error:
```rust
pub async fn force_refresh_anthropic_oauth(&self) -> std::result::Result<(), RefreshTokenError> {
    let auth = self.auth_cached_for_provider(ProviderName::Anthropic);
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

    let now = chrono::Utc::now().timestamp();
    let expires_at = now.saturating_add(i64::try_from(tokens.expires_in).unwrap_or(3600));
    // Same persist-then-reload pattern: only reload after successful save.
    let storage = create_auth_storage(self.orbit_code_home.clone(), self.auth_credentials_store_mode);
    match storage.load() {
        Ok(Some(mut v2)) => {
            v2.set_provider_auth(ProviderName::Anthropic, ProviderAuth::AnthropicOAuth {
                access_token: tokens.access_token,
                refresh_token: tokens.refresh_token,
                expires_at,
            });
            storage.save(&v2).map_err(RefreshTokenError::Transient)?;
            self.reload();
            Ok(())
        }
        Ok(None) => {
            tracing::warn!("Force refresh succeeded but no auth storage; keeping cached token");
            Err(RefreshTokenError::Transient(std::io::Error::other("no auth storage after refresh")))
        }
        Err(e) => {
            tracing::warn!("Force refresh succeeded but storage unreadable: {e}");
            Err(RefreshTokenError::Transient(e))
        }
    }
}
```

### 3d: Coherent refresh ownership model

| Scenario | Layer | Mechanism |
|----------|-------|-----------|
| Token expiring within 5 min, request about to be made | `stream_anthropic_messages()` | `maybe_refresh_anthropic_oauth()` — async, best-effort, before sync auth resolution |
| 401 response from API | `UnauthorizedRecovery` | `AnthropicOAuth` mode — reload from storage, then refresh, then done |
| Token refreshed on disk by another process | `UnauthorizedRecovery::Reload` step | Simple `manager.reload()` without account-id guard |
| Refresh succeeds but request uses stale in-memory token | Cannot happen | Proactive refresh calls `manager.reload()` before `resolve_anthropic_auth()` reads the cache; 401 recovery also reloads before retry |

---

## Step 4: Add OAuth to TUI with extracted modules

_(Unchanged from v2 — all substeps 4a through 4f remain the same.)_

### 4a: Extract Anthropic OAuth helper modules

Following `headless_chatgpt_login.rs` pattern:

**New file:** `tui/src/onboarding/auth/anthropic_oauth.rs`
**New file:** `tui_app_server/src/onboarding/auth/anthropic_oauth.rs`

### 4b: Direct TUI variant — uses `AuthManager`, saves via `save_auth_v2()`, attempt-matching guard

### 4c: App-server TUI variant — uses existing `LoginAccountParams::AnthropicOAuth` and `SubmitOAuthCode` RPCs, does NOT write auth directly

### 4d: New `SignInState` variants: `AnthropicPickMethod`, `AnthropicOAuthCodeEntry(...)`, `AnthropicOAuthSuccess`

### 4e: Update quit suppression for `AnthropicApiKeyEntry(_)` and `AnthropicOAuthCodeEntry(_)`

### 4f: Extend `cancel_login_v2` for Anthropic OAuth entries in `oauth_login_manager`

---

## Step 5: Validate pasted state

**File:** `login/src/anthropic.rs`

In `anthropic_exchange_code()`, validate pasted `state` against `verifier`. Return a dedicated error:

```rust
#[error("Authorization code is from a different or expired login attempt. Please restart the sign-in flow.")]
StaleAttempt,
```

```rust
if let Some(ref pasted_state) = state {
    if pasted_state != verifier {
        return Err(AnthropicLoginError::StaleAttempt);
    }
}
```

The TUI can display this message directly — it tells the user to restart rather than just "invalid code".

---

## Step 6: Verification

### Targeted tests to add
```
anthropic/src/client.rs:          AnthropicStream abort-on-drop behavior
login/src/anthropic.rs:           anthropic_exchange_code() state mismatch → StaleAttempt
app-server/tests/suite/v2/:       account/login/start for AnthropicOAuth
app-server/tests/suite/v2/:       account/oauth/submitCode success + failure
app-server/tests/suite/v2/:       account/login/cancel clearing pending OAuth entries
```

### Automated test suite
```bash
cd codex-rs
cargo test -p orbit-code-anthropic
cargo test -p orbit-code-login
cargo test -p orbit-code-core -- auth
cargo test -p orbit-code-app-server
cargo test -p orbit-code-tui
cargo test -p orbit-code-tui-app-server
cargo insta pending-snapshots -p orbit-code-tui
cargo insta pending-snapshots -p orbit-code-tui-app-server
```

### Interactive smoke tests (tmux, from `codex-rs/`)
```bash
cd codex-rs && cargo build --bin orbit-code && cargo run --bin orbit-code -- logout

# Test 1: API key sign-in → message → interrupt → exit
# Test 2: OAuth sign-in → browser opens → paste code → success
# Test 3: Esc during OAuth code entry → return to sub-menu
# Test 4: Esc during sub-menu → return to main provider list
# Test 5: Send message → verify response → Esc interrupt works → Ctrl+C exits
```

---

## Files Modified

| File | Change |
|------|--------|
| `core/src/default_client.rs` | Add `.connect_timeout(10s)` |
| `login/src/anthropic.rs` | CA-aware client, `ClientBuild`/`StaleAttempt` error variants, state validation |
| `anthropic/src/client.rs` | `JoinHandle` abort-on-drop for `AnthropicStream` |
| `anthropic/src/token_refresh.rs` | **New** — `refresh_anthropic_token()` (no crate cycle) |
| `anthropic/src/lib.rs` | Export `refresh_anthropic_token`, `RefreshedTokens` |
| `core/src/auth.rs` | `AnthropicOAuth` mode in `UnauthorizedRecovery`, `refresh_anthropic_oauth_if_needed()`, `force_refresh_anthropic_oauth()` |
| `core/src/client.rs` | Call `manager.refresh_anthropic_oauth_if_needed()` before sync auth resolution |
| `tui/src/onboarding/auth.rs` | Add `AnthropicPickMethod`, OAuth states, delegate to extracted module |
| `tui/src/onboarding/auth/anthropic_oauth.rs` | **New** — extracted OAuth flow for direct TUI |
| `tui/src/onboarding/onboarding_screen.rs` | Quit suppression for Anthropic input states |
| `tui_app_server/src/onboarding/auth.rs` | Same states, delegate to extracted module |
| `tui_app_server/src/onboarding/auth/anthropic_oauth.rs` | **New** — extracted OAuth flow via app-server RPCs |
| `tui_app_server/src/onboarding/onboarding_screen.rs` | Quit suppression for Anthropic input states |
| `app-server/src/orbit_code_message_processor.rs` | Extend `cancel_login_v2` for Anthropic OAuth |

## Edge Cases Handled

| Edge Case | Resolution |
|-----------|------------|
| OAuth token expires in later session | `maybe_refresh_anthropic_oauth()` refreshes proactively before request; `UnauthorizedRecovery` handles 401 reactively |
| Refresh succeeds but request path uses stale token | Cannot happen: `manager.reload()` runs inside `AuthManager` before `resolve_anthropic_auth()` reads cache |
| Storage missing/unreadable but cached OAuth present | Proactive: logs warning, skips reload, proceeds with cached token. Force (401 recovery): returns `Transient` error, keeps cached token intact, surfaces error to retry logic |
| HTTP refresh succeeds but save fails | Only `reload()` on successful save. On save failure: log, keep cached token, return error from force variant |
| Reload step finds token changed on disk | Treated as success for that retry cycle (another process refreshed); still advances to `Done` if unchanged |
| User starts OAuth, Esc, starts again before exchange finishes | Attempt-matching guard in exchange task; stale task's state update is a no-op |
| App-server Esc during Anthropic OAuth | `cancel_login_v2` extended to clear pending entries from `oauth_login_manager` |
| Stale `{code}#{state}` from different attempt | `StaleAttempt` error with user-facing "restart the sign-in flow" message |
| User presses `q` during Anthropic input states | Quit suppression protects `AnthropicApiKeyEntry` and `AnthropicOAuthCodeEntry` |
| Anthropic login client bypasses CA certs | Fixed: `build_anthropic_login_client()` uses CA-aware builder with timeouts |
| `UnauthorizedRecovery` skips for Anthropic (no account id) | New `AnthropicOAuth` recovery mode skips account-id guard, goes straight to reload → refresh |
| `codex-core` → `codex-login` crate cycle | Fixed: refresh logic in `orbit-code-anthropic` (valid dependency direction) |
| Reload step finds token changed on disk — stop early or continue? | If reload detects a different token (another process refreshed), return `auth_state_changed: Some(true)` and advance to `RefreshToken` step. The retry cycle will use the new token; if it still gets 401, the `RefreshToken` step force-refreshes. This matches the ChatGPT managed-auth pattern where reload-changed skips to next step. |
| `force_refresh_anthropic_oauth()` error typing | Uses `RefreshTokenError` (same as ChatGPT refresh). `Permanent` for non-recoverable (e.g., refresh token revoked — API returns 400 with `invalid_grant`). `Transient` for network/storage failures. No new error type needed — the existing enum covers both cases. |
| Telemetry for "stale attempt code pasted" vs general OAuth failure | `StaleAttempt` error from `anthropic_exchange_code()` is logged with `tracing::warn!("anthropic_oauth_stale_attempt")` tag. General exchange failures logged as `tracing::warn!("anthropic_oauth_exchange_failed")`. TUI shows different user-facing messages: stale → "restart the sign-in flow", other → the API error message. No new telemetry counters — rely on structured log events for now. |
