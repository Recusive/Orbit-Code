# Multi-Provider Auth Switching Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let users switch between API key and OAuth authentication for both OpenAI and Anthropic providers, mid-session via `/model` and a new `/auth` command.

**Architecture:** TUI-only approach. The existing V2 auth storage already supports multi-provider credentials. We add a `preferred_auth_modes` field to `AuthDotJsonV2`, a third popup step in the model-switch flow, a new `/auth` slash command, and mirror everything in `tui_app_server`.

**Tech Stack:** Rust, ratatui, orbit-code-core auth module, orbit-code-tui SelectionView popups.

**Spec:** `docs/superpowers/specs/2026-03-21-multi-provider-auth-switching-design.md`

---

## File Structure

| File | Responsibility |
|------|---------------|
| `core/src/auth/storage.rs` | Add `preferred_auth_modes` to `AuthDotJsonV2` |
| `core/src/auth/storage_tests.rs` | Tests for preferred_auth_modes serialization |
| `core/src/auth/persistence.rs` | Merge-on-save for preferred_auth_modes |
| `core/src/auth/persistence_tests.rs` | Tests for preference persistence |
| `tui/src/slash_command.rs` | Add `Auth` variant |
| `tui/src/chatwidget.rs` | `open_auth_popup()`, `on_slash_auth()`, modify `apply_model_and_effort()` |
| `tui/src/chatwidget/tests.rs` | Snapshot tests for auth popups |
| `tui_app_server/src/slash_command.rs` | Mirror: Add `Auth` variant |
| `tui_app_server/src/chatwidget.rs` | Mirror: same auth popup logic |
| `tui_app_server/src/chatwidget/tests.rs` | Mirror: same snapshot tests |

---

### Task 1: Add `preferred_auth_modes` to `AuthDotJsonV2`

**Files:**
- Modify: `codex-rs/core/src/auth/storage.rs:99-138`
- Modify: `codex-rs/core/src/auth/storage_tests.rs`

- [ ] **Step 1: Write failing test for preferred_auth_modes serialization**

In `storage_tests.rs`, add a test that creates an `AuthDotJsonV2` with a `preferred_auth_modes` entry, serializes to JSON, deserializes, and asserts the preference round-trips.

```rust
#[test]
fn preferred_auth_modes_round_trips() {
    let mut v2 = AuthDotJsonV2::new();
    v2.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicApiKey { key: "sk-ant-test".to_string() },
    );
    v2.set_preferred_auth_mode(ProviderName::Anthropic, AuthMode::AnthropicApiKey);

    let json = serde_json::to_string(&v2).expect("serialize");
    let deserialized: AuthDotJsonV2 = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(
        deserialized.preferred_auth_mode(ProviderName::Anthropic),
        Some(AuthMode::AnthropicApiKey)
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p orbit-code-core -- storage_tests::preferred_auth_modes_round_trips`
Expected: FAIL — `set_preferred_auth_mode` and `preferred_auth_mode` don't exist yet.

- [ ] **Step 3: Add `preferred_auth_modes` field and methods to `AuthDotJsonV2`**

In `storage.rs`, modify `AuthDotJsonV2`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDotJsonV2 {
    pub version: u32,
    pub providers: HashMap<ProviderName, ProviderAuth>,
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
        preferred_auth_modes: HashMap::new(),
    }
}
```

Add methods:

```rust
pub fn preferred_auth_mode(&self, provider: ProviderName) -> Option<AuthMode> {
    self.preferred_auth_modes.get(&provider).copied()
}

pub fn set_preferred_auth_mode(&mut self, provider: ProviderName, mode: AuthMode) {
    self.preferred_auth_modes.insert(provider, mode);
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p orbit-code-core -- storage_tests::preferred_auth_modes_round_trips`
Expected: PASS

- [ ] **Step 5: Write test for backward compatibility (v2 without preferred_auth_modes)**

```rust
#[test]
fn v2_without_preferred_auth_modes_deserializes() {
    let json = r#"{"version":2,"providers":{}}"#;
    let v2: AuthDotJsonV2 = serde_json::from_str(json).expect("deserialize");
    assert!(v2.preferred_auth_modes.is_empty());
}
```

- [ ] **Step 6: Run test to verify it passes**

Run: `cargo test -p orbit-code-core -- storage_tests::v2_without_preferred_auth_modes`
Expected: PASS (due to `#[serde(default)]`)

- [ ] **Step 7: Run `just fmt` and commit**

```bash
just fmt
git add codex-rs/core/src/auth/storage.rs codex-rs/core/src/auth/storage_tests.rs
git commit -m "feat(auth): add preferred_auth_modes to AuthDotJsonV2"
```

---

### Task 2: Merge `preferred_auth_modes` on save

**Files:**
- Modify: `codex-rs/core/src/auth/persistence.rs:112-130`
- Modify: `codex-rs/core/src/auth/persistence_tests.rs`

- [ ] **Step 1: Write failing test for preference merge-on-save**

```rust
#[test]
fn save_auth_v2_merges_preferred_auth_modes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let home = dir.path();

    // Save initial with OpenAI preference
    let mut initial = AuthDotJsonV2::new();
    initial.set_provider_auth(
        ProviderName::OpenAI,
        ProviderAuth::OpenAiApiKey { key: "sk-test".to_string() },
    );
    initial.set_preferred_auth_mode(ProviderName::OpenAI, AuthMode::ApiKey);
    save_auth_v2(home, &initial, AuthCredentialsStoreMode::File).expect("save initial");

    // Save update with Anthropic preference (should preserve OpenAI preference)
    let mut update = AuthDotJsonV2::new();
    update.set_provider_auth(
        ProviderName::Anthropic,
        ProviderAuth::AnthropicApiKey { key: "sk-ant-test".to_string() },
    );
    update.set_preferred_auth_mode(ProviderName::Anthropic, AuthMode::AnthropicApiKey);
    save_auth_v2(home, &update, AuthCredentialsStoreMode::File).expect("save update");

    // Load and verify both preferences preserved
    let loaded = load_auth_dot_json_v2(home, AuthCredentialsStoreMode::File)
        .expect("load")
        .expect("some");
    assert_eq!(loaded.preferred_auth_mode(ProviderName::OpenAI), Some(AuthMode::ApiKey));
    assert_eq!(loaded.preferred_auth_mode(ProviderName::Anthropic), Some(AuthMode::AnthropicApiKey));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p orbit-code-core -- persistence_tests::save_auth_v2_merges_preferred`
Expected: FAIL — `save_auth_v2` doesn't merge `preferred_auth_modes` yet.

- [ ] **Step 3: Update `save_auth_v2` to merge preferences**

In `persistence.rs`, inside `save_auth_v2`, add after the provider merge loop:

```rust
// Merge preferred_auth_modes
for (provider, mode) in &auth.preferred_auth_modes {
    existing.set_preferred_auth_mode(*provider, *mode);
}
```

Also do the same in `save_auth` (the v1-compat path) if it touches v2 storage.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p orbit-code-core -- persistence_tests::save_auth_v2_merges_preferred`
Expected: PASS

- [ ] **Step 5: Run `just fmt` and commit**

```bash
just fmt
git add codex-rs/core/src/auth/persistence.rs codex-rs/core/src/auth/persistence_tests.rs
git commit -m "feat(auth): merge preferred_auth_modes on save_auth_v2"
```

---

### Task 3: Add `/auth` slash command

**Files:**
- Modify: `codex-rs/tui/src/slash_command.rs:12-180`
- Modify: `codex-rs/tui_app_server/src/slash_command.rs` (mirror)

- [ ] **Step 1: Add `Auth` variant to `SlashCommand` enum**

In `tui/src/slash_command.rs`, add `Auth` after `Model` in the enum (they're grouped by frequency):

```rust
pub enum SlashCommand {
    Model,
    Auth,  // NEW — manage authentication
    // ... rest unchanged
}
```

- [ ] **Step 2: Add description**

In `description()` match:

```rust
SlashCommand::Auth => "manage authentication for model providers",
```

- [ ] **Step 3: Add `available_during_task` — false**

In `available_during_task()`, add `SlashCommand::Auth` to the `false` arm alongside `SlashCommand::Model`.

- [ ] **Step 4: Mirror in `tui_app_server/src/slash_command.rs`**

Apply identical changes.

- [ ] **Step 5: Run `just fmt` and commit**

```bash
just fmt
git add codex-rs/tui/src/slash_command.rs codex-rs/tui_app_server/src/slash_command.rs
git commit -m "feat(tui): add /auth slash command variant"
```

---

### Task 4: Implement `open_auth_popup()` in chatwidget

**Files:**
- Modify: `codex-rs/tui/src/chatwidget.rs`

This is the core UI logic. The auth popup builds context-aware items based on what credentials exist for the target provider.

- [ ] **Step 1: Add helper to detect provider from model slug**

```rust
fn provider_for_model(slug: &str) -> ProviderName {
    if slug.starts_with("claude-") {
        ProviderName::Anthropic
    } else {
        ProviderName::OpenAI
    }
}
```

- [ ] **Step 2: Add helper to mask credential strings**

```rust
fn mask_credential(value: &str, visible_prefix: usize, visible_suffix: usize) -> String {
    if value.len() <= visible_prefix + visible_suffix {
        return "*".repeat(value.len());
    }
    let prefix = &value[..visible_prefix];
    let suffix = &value[value.len() - visible_suffix..];
    format!("{prefix}{}{}suffix", "*".repeat(7))
}
```

- [ ] **Step 3: Implement `open_auth_popup()`**

Add a new method to `ChatWidget` that:
1. Takes `target_provider: ProviderName`, `model: String`, `effort: Option<ReasoningEffortConfig>`, `is_standalone: bool` (true for `/auth`, false for model-switch)
2. Loads existing credentials for the provider via `AuthManager::auth_cached_for_provider()`
3. Loads `preferred_auth_mode` from v2 storage
4. Builds `SelectionItem` list:
   - If API key exists: "API Key (current) sk-ant-***dk3F"
   - "Enter new API Key"
   - If OAuth exists: "OAuth (current) signed in as ..."
   - "OAuth Login"
   - If standalone (`/auth`): "Remove credentials"
5. Pre-highlights the item matching `preferred_auth_mode`
6. On selection:
   - Existing credential selected → save preference, apply model switch (if not standalone)
   - "Enter new API Key" → open inline text input popup (Task 5)
   - "OAuth Login" → open OAuth sub-menu popup (Task 6)
   - "Remove credentials" → confirm and call `logout_provider()`

Use the same `SelectionViewParams` pattern as `open_reasoning_popup()`.

- [ ] **Step 4: Wire auth popup into model-switch flow**

Modify the effort popup's selection handler. Currently after effort selection it calls `apply_model_and_effort()`. Change to:

```rust
// After effort is selected:
let target_provider = provider_for_model(&selected_model);
let current_provider = provider_for_model(self.current_model());

if target_provider != current_provider {
    // Open auth popup before applying
    self.open_auth_popup(target_provider, selected_model, selected_effort, /*is_standalone*/ false);
} else {
    self.apply_model_and_effort(selected_model, selected_effort);
}
```

- [ ] **Step 5: Wire `/auth` command handler**

Add `on_slash_auth()` that shows a provider selection popup first:

```rust
pub(crate) fn on_slash_auth(&mut self) {
    // Show provider picker: "Manage OpenAI" / "Manage Anthropic"
    // On selection, call open_auth_popup(provider, current_model, None, true)
}
```

Wire it in the slash command dispatch (same place `/model`, `/status` are handled).

- [ ] **Step 6: Run `just fmt`**

```bash
just fmt
```

- [ ] **Step 7: Commit**

```bash
git add codex-rs/tui/src/chatwidget.rs
git commit -m "feat(tui): implement auth popup and wire into model-switch flow"
```

---

### Task 5: Implement inline API key input popup

**Files:**
- Modify: `codex-rs/tui/src/chatwidget.rs`

- [ ] **Step 1: Implement `open_api_key_input_popup()`**

This method opens an inline text input view where the user pastes an API key. Use the same input infrastructure as the existing composer but in a popup context.

Key behaviors:
- Input is masked with `*` characters as the user types
- Basic format validation on Enter:
  - Anthropic: starts with `sk-ant-`
  - OpenAI: starts with `sk-`
- On validation failure: show inline error, let user retry
- On success: save via `save_auth_v2()`, save preference, then apply model switch

Parameters: `provider: ProviderName`, `model: String`, `effort: Option<ReasoningEffortConfig>`, `is_standalone: bool`

- [ ] **Step 2: Wire from auth popup selection**

When user selects "Enter new API Key" in `open_auth_popup()`, call `open_api_key_input_popup()`.

- [ ] **Step 3: Run `just fmt` and commit**

```bash
just fmt
git add codex-rs/tui/src/chatwidget.rs
git commit -m "feat(tui): add inline API key input popup with masking"
```

---

### Task 6: Implement OAuth sub-menu and token paste popup

**Files:**
- Modify: `codex-rs/tui/src/chatwidget.rs`

- [ ] **Step 1: Implement `open_oauth_submenu_popup()`**

Shows two options:
1. "Browser Login" — triggers existing headless OAuth flow
2. "Paste Token" — opens inline token input

Parameters: `provider: ProviderName`, `model: String`, `effort: Option<ReasoningEffortConfig>`, `is_standalone: bool`

- [ ] **Step 2: Implement `open_oauth_token_input_popup()`**

Same pattern as API key input but for OAuth tokens:
- Input masked with `*`
- No format validation (OAuth tokens vary)
- On Enter: save as `ProviderAuth::AnthropicOAuth` or `ProviderAuth::Chatgpt` with appropriate fields
- Save preference, apply model switch

- [ ] **Step 3: Wire browser OAuth flow**

When user selects "Browser Login":
- For OpenAI: call existing `headless_chatgpt_login` flow
- For Anthropic: call existing Anthropic OAuth flow (if exists) or show "not yet supported" message

- [ ] **Step 4: Run `just fmt` and commit**

```bash
just fmt
git add codex-rs/tui/src/chatwidget.rs
git commit -m "feat(tui): add OAuth sub-menu with browser login and token paste"
```

---

### Task 7: Implement `/auth` status view

**Files:**
- Modify: `codex-rs/tui/src/chatwidget.rs`

- [ ] **Step 1: Implement auth status display in `on_slash_auth()`**

Build a `StatusHistoryCell`-style display showing:
- Per provider: auth method + masked credential summary
- Selection items: "Manage OpenAI" / "Manage Anthropic"

Use the same `SelectionViewParams` pattern. On selection, delegate to `open_auth_popup()` with `is_standalone: true`.

- [ ] **Step 2: Add "Remove credentials" confirmation**

When user selects "Remove credentials" in the standalone auth popup:
- Show confirmation: "Remove all [Provider] credentials? This cannot be undone."
- Two options: "Yes, remove" / "Cancel"
- On confirm: call `logout_provider()`, show success message

- [ ] **Step 3: Run `just fmt` and commit**

```bash
just fmt
git add codex-rs/tui/src/chatwidget.rs
git commit -m "feat(tui): implement /auth status view with provider management"
```

---

### Task 8: Snapshot tests for auth popups

**Files:**
- Modify: `codex-rs/tui/src/chatwidget/tests.rs`

- [ ] **Step 1: Write snapshot test — auth popup with no credentials**

```rust
#[tokio::test]
async fn auth_popup_no_credentials_snapshot() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(Some("gpt-5.3-codex")).await;
    chat.open_auth_popup(ProviderName::Anthropic, "claude-opus-4-6".to_string(), None, false);
    let popup = render_bottom_popup(&chat, 80);
    assert_snapshot!("auth_popup_no_credentials", popup);
}
```

- [ ] **Step 2: Write snapshot test — auth popup with existing API key**

Similar test but pre-populate Anthropic API key auth in the chat's auth manager before opening the popup.

- [ ] **Step 3: Write snapshot test — auth popup with existing OAuth**

Pre-populate Anthropic OAuth auth, verify "(current)" marker appears.

- [ ] **Step 4: Write snapshot test — /auth status view**

```rust
#[tokio::test]
async fn auth_status_view_snapshot() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(Some("gpt-5.3-codex")).await;
    // Pre-populate both providers
    chat.on_slash_auth();
    let popup = render_bottom_popup(&chat, 80);
    assert_snapshot!("auth_status_view", popup);
}
```

- [ ] **Step 5: Run tests and accept snapshots**

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

### Task 9: Mirror all TUI changes in `tui_app_server`

**Files:**
- Modify: `codex-rs/tui_app_server/src/chatwidget.rs`
- Modify: `codex-rs/tui_app_server/src/chatwidget/tests.rs`

Per convention 54, all UI changes must be mirrored.

- [ ] **Step 1: Copy `provider_for_model()` and `mask_credential()` helpers**

- [ ] **Step 2: Copy `open_auth_popup()` implementation**

- [ ] **Step 3: Copy `open_api_key_input_popup()` implementation**

- [ ] **Step 4: Copy `open_oauth_submenu_popup()` and `open_oauth_token_input_popup()`**

- [ ] **Step 5: Copy `on_slash_auth()` and auth status view**

- [ ] **Step 6: Wire `/auth` in slash command dispatch**

- [ ] **Step 7: Copy all snapshot tests**

- [ ] **Step 8: Run tests and accept snapshots**

```bash
cargo insta test -p orbit-code-tui-app-server --accept
```

- [ ] **Step 9: Run `just fmt` and commit**

```bash
just fmt
git add codex-rs/tui_app_server/src/
git commit -m "feat(tui_app_server): mirror auth popup changes from tui"
```

---

### Task 10: Regenerate config schema and final validation

**Files:**
- Run schema generation
- Run full test suites

- [ ] **Step 1: Regenerate config schema**

```bash
just write-config-schema
```

- [ ] **Step 2: Run clippy on changed crates**

```bash
just fix -p orbit-code-core
just fix -p orbit-code-tui
just fix -p orbit-code-tui-app-server
```

- [ ] **Step 3: Run tests on changed crates**

```bash
cargo test -p orbit-code-core
cargo test -p orbit-code-tui
cargo test -p orbit-code-tui-app-server
```

- [ ] **Step 4: Run `just fmt`**

```bash
just fmt
```

- [ ] **Step 5: Final commit if any schema/lint changes**

```bash
git add -A
git commit -m "chore: regenerate config schema, fix lint"
```
