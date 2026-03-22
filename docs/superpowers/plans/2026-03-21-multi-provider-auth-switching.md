# Multi-Provider Auth Switching Implementation Plan (v3 — Final)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let users switch between API key and OAuth authentication for both OpenAI and Anthropic providers mid-session, via a third step in the `/model` flow and a new `/auth` command.

**Architecture:** Extend the existing V2 auth storage with two optional backward-compatible fields (`alternate_credentials`, `preferred_auth_modes`). No format version change. All existing writers (onboarding, CLI, app-server, refresh) continue working unchanged because they go through `save_auth`/`save_auth_v2` which does load-merge-save. Auth popup lives in a new `chatwidget/auth_popup.rs` submodule. Phase 1 covers standalone `tui` only.

**Tech Stack:** Rust, ratatui, orbit-code-core auth module, orbit-code-tui SelectionView popups.

**Spec:** `docs/superpowers/specs/2026-03-21-multi-provider-auth-switching-design.md`

---

## File Structure

| File | Responsibility |
|------|---------------|
| `core/src/auth/storage.rs` | Add `alternate_credentials`, `preferred_auth_modes` to `AuthDotJsonV2`. Swap helpers. Backup. |
| `core/src/auth/storage_tests.rs` | Round-trip, backward compat, swap tests |
| `core/src/auth/persistence.rs` | Merge logic for new fields. `swap_auth_method()` helper. |
| `core/src/auth_tests.rs` | `preferred_mode` resolution tests |
| `core/src/auth/manager.rs` | `auth_cached_for_provider()` respects `preferred_auth_modes` |
| `tui/src/slash_command.rs` | Add `Auth` variant |
| `tui/src/chatwidget/auth_popup.rs` | NEW: auth popup logic, API key input, provider detection |
| `tui/src/chatwidget.rs` | Wire auth step into model-switch, wire `/auth` command |
| `tui/src/chatwidget/tests.rs` | Snapshot tests for auth popups |

---

### Task 1: Add `alternate_credentials` and `preferred_auth_modes` to `AuthDotJsonV2`

**Files:**
- Modify: `codex-rs/core/src/auth/storage.rs:99-138`
- Modify: `codex-rs/core/src/auth/storage_tests.rs`

- [ ] **Step 1: Write failing test for new fields round-trip**

In `storage_tests.rs`:

```rust
#[test]
fn alternate_credentials_and_preferred_modes_round_trip() {
    let mut v2 = AuthDotJsonV2::new();
    v2.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicApiKey { key: "sk-ant-active".to_string() },
    );
    v2.alternate_credentials.insert(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicOAuth {
            access_token: "at".to_string(),
            refresh_token: "rt".to_string(),
            expires_at: 999,
        },
    );
    v2.preferred_auth_modes.insert(ProviderName::Anthropic, AuthMode::AnthropicApiKey);

    let json = serde_json::to_string_pretty(&v2).expect("serialize");
    let loaded = deserialize_auth(&json).expect("deserialize");

    assert_eq!(loaded.provider_auth(ProviderName::Anthropic), v2.provider_auth(ProviderName::Anthropic));
    assert_eq!(loaded.alternate_credentials.get(&ProviderName::Anthropic), v2.alternate_credentials.get(&ProviderName::Anthropic));
    assert_eq!(loaded.preferred_auth_modes.get(&ProviderName::Anthropic), Some(&AuthMode::AnthropicApiKey));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p orbit-code-core -- storage_tests::alternate_credentials_and_preferred_modes_round_trip`
Expected: FAIL — fields don't exist yet.

- [ ] **Step 3: Add new fields to `AuthDotJsonV2`**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDotJsonV2 {
    pub version: u32,
    pub providers: HashMap<ProviderName, ProviderAuth>,

    /// Stored-but-inactive credential per provider. When user switches
    /// auth method, the old credential moves here from `providers`.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub alternate_credentials: HashMap<ProviderName, ProviderAuth>,

    /// Last-used auth method per provider. Determines pre-highlight
    /// in the auth popup and credential resolution order.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub preferred_auth_modes: HashMap<ProviderName, AuthMode>,
}
```

Update `AuthDotJsonV2::new()`:

```rust
pub fn new() -> Self {
    Self {
        version: 2,
        providers: HashMap::new(),
        alternate_credentials: HashMap::new(),
        preferred_auth_modes: HashMap::new(),
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p orbit-code-core -- storage_tests::alternate_credentials_and_preferred_modes_round_trip`
Expected: PASS

- [ ] **Step 5: Update `set_provider_auth()` to auto-preserve on method change only**

This is the critical change. When a credential with a DIFFERENT auth method type is written, the old one auto-moves to `alternate_credentials`. Same-method rewrites (e.g., OAuth refresh) replace in place without touching alternate:

```rust
pub fn set_provider_auth(&mut self, provider: ProviderName, auth: ProviderAuth) {
    if let Some(old) = self.providers.insert(provider, auth) {
        // Only preserve as alternate when the auth method type changes.
        // Same-method rewrites (OAuth refresh, key rotation) replace in place.
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

- [ ] **Step 6: Write test for cross-method auto-preserve**

```rust
#[test]
fn set_provider_auth_preserves_old_on_method_switch() {
    let mut v2 = AuthDotJsonV2::new();
    v2.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicApiKey { key: "sk-ant-old".to_string() },
    );
    v2.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicOAuth {
            access_token: "at".to_string(),
            refresh_token: "rt".to_string(),
            expires_at: 999,
        },
    );

    // Active should be OAuth
    assert!(matches!(v2.provider_auth(ProviderName::Anthropic), Some(ProviderAuth::AnthropicOAuth { .. })));
    // Old API key should be in alternate
    assert!(matches!(v2.alternate_credentials.get(&ProviderName::Anthropic), Some(ProviderAuth::AnthropicApiKey { .. })));
}
```

- [ ] **Step 7: Write test for same-method rewrite NOT touching alternate**

```rust
#[test]
fn set_provider_auth_same_method_rewrite_preserves_alternate() {
    let mut v2 = AuthDotJsonV2::new();
    // Set API key, then switch to OAuth (API key moves to alternate)
    v2.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicApiKey { key: "sk-ant-key".to_string() },
    );
    v2.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicOAuth {
            access_token: "at-old".to_string(),
            refresh_token: "rt".to_string(),
            expires_at: 999,
        },
    );
    // Now simulate OAuth refresh — same method type, new token
    v2.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicOAuth {
            access_token: "at-refreshed".to_string(),
            refresh_token: "rt-new".to_string(),
            expires_at: 2000,
        },
    );

    // Active should be refreshed OAuth
    if let Some(ProviderAuth::AnthropicOAuth { access_token, .. }) = v2.provider_auth(ProviderName::Anthropic) {
        assert_eq!(access_token, "at-refreshed");
    } else {
        panic!("expected AnthropicOAuth");
    }
    // Alternate should still be the original API key (NOT the old OAuth)
    assert!(matches!(
        v2.alternate_credentials.get(&ProviderName::Anthropic),
        Some(ProviderAuth::AnthropicApiKey { .. })
    ));
}
```

- [ ] **Step 8: Run tests to verify both pass**

Expected: PASS

- [ ] **Step 9: Write backward-compat test (V2 without new fields)**

```rust
#[test]
fn v2_without_new_fields_deserializes_with_empty_defaults() {
    let json = r#"{"version":2,"providers":{}}"#;
    let v2 = deserialize_auth(json).expect("deserialize");
    assert!(v2.alternate_credentials.is_empty());
    assert!(v2.preferred_auth_modes.is_empty());
}
```

- [ ] **Step 10: Run test to verify it passes**

Expected: PASS (due to `#[serde(default)]`).

- [ ] **Step 11: Run `just fmt` and commit**

```bash
just fmt
git add codex-rs/core/src/auth/storage.rs codex-rs/core/src/auth/storage_tests.rs
git commit -m "feat(auth): add alternate_credentials, preferred_auth_modes, and discriminant-aware auto-preserve to AuthDotJsonV2"
```

---

### Task 2: Add `restore_alternate_credential` and `remove_all_credentials` helpers

**Files:**
- Modify: `codex-rs/core/src/auth/storage.rs`
- Modify: `codex-rs/core/src/auth/storage_tests.rs`

Note: `set_provider_auth()` (updated in Task 1) already handles the forward swap — writing a new credential auto-preserves the old one. This task adds the reverse operation (restore alternate as active) and full removal.

- [ ] **Step 1: Write failing test for restore_alternate**

```rust
#[test]
fn restore_alternate_swaps_back() {
    let mut v2 = AuthDotJsonV2::new();
    // First set: API key becomes active
    v2.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicApiKey { key: "sk-ant-key".to_string() },
    );
    // Second set: OAuth becomes active, API key auto-moves to alternate
    v2.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicOAuth {
            access_token: "at".to_string(),
            refresh_token: "rt".to_string(),
            expires_at: 999,
        },
    );
    // Restore: API key should become active again, OAuth moves to alternate
    v2.restore_alternate_credential(ProviderName::Anthropic);

    assert!(matches!(
        v2.provider_auth(ProviderName::Anthropic),
        Some(ProviderAuth::AnthropicApiKey { .. })
    ));
    assert!(matches!(
        v2.alternate_credentials.get(&ProviderName::Anthropic),
        Some(ProviderAuth::AnthropicOAuth { .. })
    ));
}
```

- [ ] **Step 2: Run test to verify it fails**

Expected: FAIL — `restore_alternate_credential` doesn't exist.

- [ ] **Step 3: Implement helpers**

Add to `impl AuthDotJsonV2`:

```rust
/// Restore the alternate credential as active. The current active
/// credential moves to alternate. Returns true if a swap occurred.
pub fn restore_alternate_credential(&mut self, provider: ProviderName) -> bool {
    if let Some(alternate) = self.alternate_credentials.remove(&provider) {
        if let Some(current) = self.providers.remove(&provider) {
            self.alternate_credentials.insert(provider, current);
        }
        self.providers.insert(provider, alternate);
        true
    } else {
        false
    }
}

/// Remove all credentials for a provider (active + alternate + preference).
pub fn remove_all_credentials(&mut self, provider: ProviderName) {
    self.providers.remove(&provider);
    self.alternate_credentials.remove(&provider);
    self.preferred_auth_modes.remove(&provider);
}
```

- [ ] **Step 4: Write test for remove_all_credentials**

```rust
#[test]
fn remove_all_credentials_clears_everything() {
    let mut v2 = AuthDotJsonV2::new();
    v2.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicApiKey { key: "sk-ant-key".to_string() },
    );
    v2.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicOAuth {
            access_token: "at".to_string(),
            refresh_token: "rt".to_string(),
            expires_at: 999,
        },
    );
    v2.preferred_auth_modes.insert(ProviderName::Anthropic, AuthMode::AnthropicOAuth);

    v2.remove_all_credentials(ProviderName::Anthropic);

    assert!(v2.provider_auth(ProviderName::Anthropic).is_none());
    assert!(v2.alternate_credentials.get(&ProviderName::Anthropic).is_none());
    assert!(v2.preferred_auth_modes.get(&ProviderName::Anthropic).is_none());
}
```

- [ ] **Step 5: Run tests, `just fmt`, commit**

```bash
just fmt
cargo test -p orbit-code-core -- storage_tests
git add codex-rs/core/src/auth/storage.rs codex-rs/core/src/auth/storage_tests.rs
git commit -m "feat(auth): add restore_alternate_credential and remove_all_credentials helpers"
```

---

### Task 3: Update persistence merge logic for new fields

**Files:**
- Modify: `codex-rs/core/src/auth/persistence.rs:89-128`
- Modify: `codex-rs/core/src/auth_tests.rs`

- [ ] **Step 1: Write failing test for merge preserving alternate_credentials**

In `auth_tests.rs`:

```rust
#[test]
fn save_auth_v2_preserves_alternate_credentials() {
    let dir = tempfile::tempdir().expect("tempdir");
    // Save initial: Anthropic API key active, OAuth as alternate
    let mut initial = AuthDotJsonV2::new();
    initial.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicApiKey { key: "sk-ant-key".to_string() },
    );
    initial.alternate_credentials.insert(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicOAuth {
            access_token: "at".to_string(),
            refresh_token: "rt".to_string(),
            expires_at: 999,
        },
    );
    initial.preferred_auth_modes.insert(ProviderName::Anthropic, AuthMode::AnthropicApiKey);
    save_auth_v2(dir.path(), &initial, AuthCredentialsStoreMode::File).expect("save");

    // Save update: only OpenAI key (should preserve Anthropic alternate + preference)
    let mut update = AuthDotJsonV2::new();
    update.set_provider_auth(
        ProviderName::OpenAI,
        ProviderAuth::OpenAiApiKey { key: "sk-openai".to_string() },
    );
    save_auth_v2(dir.path(), &update, AuthCredentialsStoreMode::File).expect("save");

    // Load and verify everything preserved
    let loaded = load_auth_dot_json_v2(dir.path(), AuthCredentialsStoreMode::File)
        .expect("load").expect("some");
    assert!(loaded.alternate_credentials.contains_key(&ProviderName::Anthropic));
    assert_eq!(loaded.preferred_auth_modes.get(&ProviderName::Anthropic), Some(&AuthMode::AnthropicApiKey));
}
```

- [ ] **Step 2: Run test to verify it fails**

Expected: FAIL — `save_auth_v2` merge loop doesn't touch `alternate_credentials` or `preferred_auth_modes`.

- [ ] **Step 3: Update merge logic in `save_auth_v2`**

In `persistence.rs`, inside `save_auth_v2`, after the existing provider merge loop, add:

```rust
// Merge alternate_credentials
for (provider, alt_auth) in &auth.alternate_credentials {
    existing.alternate_credentials.insert(*provider, alt_auth.clone());
}
// Merge preferred_auth_modes
for (provider, mode) in &auth.preferred_auth_modes {
    existing.preferred_auth_modes.insert(*provider, *mode);
}
```

Also update `save_auth` (the v1 compat path) — the v1->v2 conversion doesn't populate new fields, so existing merge-on-save naturally preserves them (the `existing` struct already has them from disk).

- [ ] **Step 4: Run test to verify it passes**

Expected: PASS

- [ ] **Step 5: Update `delete_provider` to clear alternate + preference**

In `storage.rs`, update `AuthStorageBackend::delete_provider()` default impl:

```rust
fn delete_provider(&self, provider: ProviderName) -> std::io::Result<bool> {
    if let Some(mut v2) = self.load()? {
        let removed_active = v2.remove_provider_auth(provider).is_some();
        let removed_alt = v2.alternate_credentials.remove(&provider).is_some();
        v2.preferred_auth_modes.remove(&provider);
        let removed = removed_active || removed_alt;
        if removed {
            if v2.has_any_auth() {
                self.save(&v2)?;
            } else {
                self.delete()?;
            }
        }
        Ok(removed)
    } else {
        Ok(false)
    }
}
```

- [ ] **Step 6: Run tests, `just fmt`, commit**

```bash
just fmt
cargo test -p orbit-code-core
git add codex-rs/core/src/auth/persistence.rs codex-rs/core/src/auth/storage.rs codex-rs/core/src/auth_tests.rs
git commit -m "feat(auth): merge alternate_credentials and preferred_auth_modes on save"
```

---

### Task 4: Update `auth_cached_for_provider()` for preferred_mode

**Files:**
- Modify: `codex-rs/core/src/auth/manager.rs:169-238`
- Modify: `codex-rs/core/src/auth_tests.rs`

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn auth_cached_for_provider_uses_alternate_when_preferred() {
    // Setup: Anthropic has API key in providers, OAuth in alternate_credentials,
    // preferred_auth_modes says AnthropicOAuth
    // Assert: auth_cached_for_provider returns OAuth, not API key
}
```

- [ ] **Step 2: Update `auth_cached_for_provider` Anthropic branch**

In the `ProviderName::Anthropic` match arm, after loading v2 from storage, check `preferred_auth_modes` and `alternate_credentials`:

```rust
ProviderName::Anthropic => {
    if let Ok(Some(v2)) =
        load_auth_dot_json_v2(&self.orbit_code_home, self.auth_credentials_store_mode)
    {
        let preferred = v2.preferred_auth_modes.get(&ProviderName::Anthropic).copied();

        // Check if preferred mode points to alternate credential
        if let Some(alt) = v2.alternate_credentials.get(&ProviderName::Anthropic)
            && preferred == Some(auth_mode_for_provider_auth(alt))
        {
            if let Some(auth) = codex_auth_from_provider_auth(alt) {
                return Some(auth);
            }
        }

        // Default: use active credential from providers
        if let Some(provider_auth) = v2.provider_auth(ProviderName::Anthropic) {
            return codex_auth_from_provider_auth(provider_auth);
        }
    }
    // Fall back to env var
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY")
        && !key.is_empty()
    {
        return Some(CodexAuth::AnthropicApiKey(AnthropicApiKeyAuth::new(key)));
    }
    None
}
```

Add helper:

```rust
fn auth_mode_for_provider_auth(auth: &ProviderAuth) -> AuthMode {
    match auth {
        ProviderAuth::OpenAiApiKey { .. } => AuthMode::ApiKey,
        ProviderAuth::Chatgpt { .. } => AuthMode::Chatgpt,
        ProviderAuth::ChatgptAuthTokens { .. } => AuthMode::ChatgptAuthTokens,
        ProviderAuth::AnthropicApiKey { .. } => AuthMode::AnthropicApiKey,
        ProviderAuth::AnthropicOAuth { .. } => AuthMode::AnthropicOAuth,
    }
}
```

- [ ] **Step 3: Run tests, `just fmt`, commit**

```bash
just fmt
cargo test -p orbit-code-core
git add codex-rs/core/src/auth/manager.rs codex-rs/core/src/auth_tests.rs
git commit -m "feat(auth): auth_cached_for_provider respects preferred_auth_modes"
```

---

### Task 5: Add `/auth` slash command

**Files:**
- Modify: `codex-rs/tui/src/slash_command.rs`

- [ ] **Step 1: Add `Auth` variant after `Model`**

Description: `"manage authentication for model providers"`. `available_during_task`: false.

- [ ] **Step 2: Run `just fmt` and commit**

```bash
just fmt
git add codex-rs/tui/src/slash_command.rs
git commit -m "feat(tui): add /auth slash command variant"
```

---

### Task 6: Create `chatwidget/auth_popup.rs` submodule

**Files:**
- Create: `codex-rs/tui/src/chatwidget/auth_popup.rs`
- Modify: `codex-rs/tui/src/chatwidget.rs` (add `mod auth_popup;`)

- [ ] **Step 1: Create `auth_popup.rs` with helpers**

```rust
//! Auth method selection popup for mid-session provider switching.

use orbit_code_core::auth::storage::ProviderName;
use orbit_code_core::auth::storage::ProviderAuth;

pub(crate) fn provider_for_model(slug: &str) -> ProviderName {
    if slug.starts_with("claude-") {
        ProviderName::Anthropic
    } else {
        ProviderName::OpenAI
    }
}

pub(crate) fn mask_credential(value: &str) -> String {
    if value.len() <= 10 {
        return "*".repeat(value.len());
    }
    let prefix = &value[..7];
    let suffix = &value[value.len() - 3..];
    format!("{prefix}*******{suffix}")
}

pub(crate) fn provider_display_name(provider: ProviderName) -> &'static str {
    match provider {
        ProviderName::OpenAI => "OpenAI",
        ProviderName::Anthropic => "Anthropic",
    }
}

pub(crate) fn credential_summary(auth: &ProviderAuth) -> String {
    match auth {
        ProviderAuth::OpenAiApiKey { key } => format!("API Key: {}", mask_credential(key)),
        ProviderAuth::Chatgpt { .. } => "OAuth (ChatGPT)".to_string(),
        ProviderAuth::ChatgptAuthTokens { .. } => "OAuth (external)".to_string(),
        ProviderAuth::AnthropicApiKey { key } => format!("API Key: {}", mask_credential(key)),
        ProviderAuth::AnthropicOAuth { .. } => "OAuth (Anthropic)".to_string(),
    }
}
```

- [ ] **Step 2: Add `open_auth_popup()` — builds context-aware selection items**

Takes `target_provider`, `model`, `effort`, `is_standalone`. Loads V2 storage. Checks `providers`, `alternate_credentials`, `preferred_auth_modes`. Builds `SelectionItem` list. Uses `SelectionViewParams` pattern.

On selection:
- Existing active credential → no change needed, apply model switch
- Existing alternate credential → call `restore_alternate_credential()`, save, apply
- "Enter new API Key" → open masked input popup
- "OAuth Login" → trigger browser OAuth flow
- "Remove credentials" (standalone only) → confirm + call `remove_all_credentials()`

- [ ] **Step 3: Add `open_api_key_input()` — masked text input popup**

Masked input. On Enter: validate format, build `ProviderAuth`, call `swap_active_credential()`, save via `save_auth_v2()`, apply model switch.

- [ ] **Step 4: Add `on_slash_auth()` — provider picker + status display**

Shows both providers with credential summaries. On selection, opens `open_auth_popup()` with `is_standalone: true`.

- [ ] **Step 5: Add `mod auth_popup;` to `chatwidget.rs`**

- [ ] **Step 6: Run `just fmt` and commit**

```bash
just fmt
git add codex-rs/tui/src/chatwidget/auth_popup.rs codex-rs/tui/src/chatwidget.rs
git commit -m "feat(tui): create auth_popup submodule with popup logic"
```

---

### Task 7: Wire auth popup into model-switch flow and `/auth` command

**Files:**
- Modify: `codex-rs/tui/src/chatwidget.rs`

- [ ] **Step 1: Modify effort popup selection handler**

After effort is selected, before `apply_model_and_effort()`:

```rust
let target_provider = auth_popup::provider_for_model(&selected_model);
let current_provider = auth_popup::provider_for_model(self.current_model());

if target_provider != current_provider {
    self.open_auth_popup(target_provider, selected_model, selected_effort, false);
} else {
    self.apply_model_and_effort(selected_model, selected_effort);
}
```

- [ ] **Step 2: Wire `/auth` in slash command dispatch**

```rust
SlashCommand::Auth => self.on_slash_auth(),
```

- [ ] **Step 3: Run `just fmt` and commit**

```bash
just fmt
git add codex-rs/tui/src/chatwidget.rs
git commit -m "feat(tui): wire auth popup into model-switch flow and /auth command"
```

---

### Task 8: Snapshot tests

**Files:**
- Modify: `codex-rs/tui/src/chatwidget/tests.rs`

- [ ] **Step 1: Write snapshot — auth popup no credentials**
- [ ] **Step 2: Write snapshot — auth popup with API key active**
- [ ] **Step 3: Write snapshot — auth popup with API key active + OAuth alternate**
- [ ] **Step 4: Write snapshot — /auth status view**
- [ ] **Step 5: Run and accept snapshots**

```bash
cargo insta test -p orbit-code-tui --accept
```

- [ ] **Step 6: Run `just fmt` and commit**

```bash
just fmt
git add codex-rs/tui/src/chatwidget/tests.rs codex-rs/tui/src/chatwidget/snapshots/
git commit -m "test(tui): add snapshot tests for auth popups"
```

---

### Task 9: Final validation

- [ ] **Step 1: Run clippy**

```bash
just fix -p orbit-code-core
just fix -p orbit-code-tui
```

- [ ] **Step 2: Run full test suites**

```bash
cargo test -p orbit-code-core
cargo test -p orbit-code-tui
```

- [ ] **Step 3: Verify no existing tests broke**

Especially: `auth_tests::save_auth_v2_preserves_existing_providers`, `storage_tests::v2_roundtrip`, and all onboarding/CLI tests that touch auth persistence.

- [ ] **Step 4: Run `just fmt`, final commit**

```bash
just fmt
git add -A
git commit -m "chore: fix lint and format after auth switching implementation"
```
