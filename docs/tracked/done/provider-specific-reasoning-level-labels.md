# Plan: Provider-Specific Reasoning Level Labels

## Context

The TUI reasoning level popup hard-codes `XHigh` as "Max" for all providers via `reasoning_effort_label()` in `chatwidget.rs`. OpenAI calls their highest level "Extra High", not "Max" — creating a visual contradiction where the label says "Max" but the description says "Extra high reasoning depth...". Claude models correctly use "Max".

**Before (gpt-5.4) — broken:**
```
4. Max               Extra high reasoning depth for complex problems
```

**After (gpt-5.4) — fixed:**
```
4. Extra High        Extra high reasoning depth for complex problems
```

**Claude Opus — unchanged:**
```
4. Max (current)     Maximum capability with no constraints. Opus only.
```

## Approach: UI-Only Provider-Aware Fix

This is a **display label issue**, not a data model issue. The underlying `XHigh` enum and all wire formats are correct — only the TUI rendering needs to change. Using the existing `auth_popup::provider_for_model()` (tui) and an identical inline check (tui_app_server) keeps the fix scoped to where the problem lives.

**Why not a schema/models.json change:** Adding `label` to `ReasoningEffortPreset` would touch the protocol crate (rebuilding ~67 crates), require schema regen, app-server-protocol updates, and `label: None` additions to ~12 test files — all for a change that only affects how one enum variant displays in two TUI files. Minimum necessary change wins here.

## Public APIs / Types

No changes. `ReasoningEffortPreset`, `ModelPreset`, app-server protocol `ReasoningEffortOption`, generated schemas, and `models.json` all remain untouched.

## Provider Detection

Both TUI crates must use the **same slug heuristic** for Anthropic detection:

```rust
slug.starts_with("claude-") || slug.starts_with("claude3")
```

This is the canonical check from `tui/src/chatwidget/auth_popup.rs:28`. The `tui_app_server` crate does not have `auth_popup`, so it uses an inline check with the **exact same condition** — not a subset. If this heuristic ever needs updating, both locations must change together.

**Default for unknown providers:** Any model slug that does not match the Anthropic heuristic gets "Extra High". This is correct because all current non-Anthropic models with `XHigh` are OpenAI models, and "Extra High" is a reasonable generic label (it describes the effort level literally). If a future provider needs a different label, a new branch can be added at that time.

## Steps

### Step 1: Add model-aware label helper in `tui` chatwidget

**File:** `codex-rs/tui/src/chatwidget.rs`

Add a private helper that delegates to `auth_popup::provider_for_model()`:

```rust
/// Returns the display label for a reasoning effort, using provider-specific
/// naming for XHigh: "Max" for Anthropic, "Extra High" for all others.
fn reasoning_effort_label_for_model(
    effort: ReasoningEffortConfig,
    model_slug: &str,
) -> &'static str {
    match effort {
        ReasoningEffortConfig::XHigh => {
            if auth_popup::provider_for_model(model_slug) == ProviderName::Anthropic {
                "Max"
            } else {
                "Extra High"
            }
        }
        other => Self::reasoning_effort_label(other),
    }
}
```

Keep `reasoning_effort_label` as the static provider-agnostic fallback for contexts without a model slug.

### Step 2: Update callsites in `open_reasoning_popup` (tui)

**File:** `codex-rs/tui/src/chatwidget.rs`

The `model_slug` is constructed at line ~7174 as `preset.model.to_string()`. Update:

- **Line ~7111** (warning text): `Self::reasoning_effort_label_for_model(effort, &preset.model)` — note: this callsite runs before `model_slug` is assigned, so use `&preset.model` directly
- **Line ~7198** (popup item label): `Self::reasoning_effort_label_for_model(effort, &model_slug)`

This ensures OpenAI shows "Extra High reasoning effort can quickly consume Plus plan rate limits." and the popup row shows "Extra High" for OpenAI, "Max" for Claude.

### Step 3: Update callsites in `open_plan_reasoning_scope_prompt` (tui)

**File:** `codex-rs/tui/src/chatwidget.rs` (lines ~7009-7042)

This method receives `model: String` and has three `reasoning_effort_label` callsites:

- **Line ~7014**: `reasoning_effort_label_for_model(selected_effort, &model)` — reasoning phrase like "extra high reasoning"
- **Line ~7025**: `reasoning_effort_label_for_model(plan_override, &model)` — plan override label
- **Line ~7032**: `reasoning_effort_label_for_model(plan_effort, &model)` — built-in plan default label

All three have access to the `model` parameter. The `.to_lowercase()` calls downstream mean "Extra High" → "extra high reasoning" reads naturally.

### Step 4: Mirror in `tui_app_server` chatwidget

**File:** `codex-rs/tui_app_server/src/chatwidget.rs`

`tui_app_server` does NOT have `auth_popup::provider_for_model()` (the auth flow is RPC-driven). Use an inline check with the **exact same heuristic** as `tui/src/chatwidget/auth_popup.rs:28`:

```rust
fn reasoning_effort_label_for_model(
    effort: ReasoningEffortConfig,
    model_slug: &str,
) -> &'static str {
    match effort {
        ReasoningEffortConfig::XHigh => {
            if model_slug.starts_with("claude-") || model_slug.starts_with("claude3") {
                "Max"
            } else {
                "Extra High"
            }
        }
        other => Self::reasoning_effort_label(other),
    }
}
```

Update the same callsites as Steps 2-3:
- `open_reasoning_popup` (lines ~7972, 8046)
- `open_plan_reasoning_scope_prompt` (lines ~7875, 7886, 7893)

### Step 5: Update existing popup tests

**File:** `codex-rs/tui/src/chatwidget/tests.rs`

- **`reasoning_popup_shows_extra_high_with_space` (line 8049):** Change assertion from `popup.contains("Max")` to `popup.contains("Extra High")`. Keep the `!popup.contains("Extrahigh")` guard. This test uses `gpt-5.1-codex-max` — should now show "Extra High", not "Max".
- **Snapshot tests** `model_reasoning_selection_popup` (line 8021) and `model_reasoning_selection_popup_extra_high_warning` (line 8035): Accept updated snapshots where "Max" → "Extra High" for OpenAI models.

**File:** `codex-rs/tui_app_server/src/chatwidget/tests.rs`

Mirror the same test changes.

### Step 6: Add Claude popup regression test

**File:** `codex-rs/tui/src/chatwidget/tests.rs`

Add a new test using `claude-opus-4-6` to assert XHigh renders as "Max" in the popup:

```rust
#[tokio::test]
async fn reasoning_popup_shows_max_for_claude_opus() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(Some("claude-opus-4-6")).await;
    // set anthropic auth...
    let preset = get_available_model(&chat, "claude-opus-4-6");
    chat.open_reasoning_popup(preset);
    let popup = render_bottom_popup(&chat, 120);
    assert!(popup.contains("Max"), "Claude Opus should show 'Max' for XHigh; popup: {popup}");
    assert!(!popup.contains("Extra High"), "Claude Opus should not show 'Extra High'; popup: {popup}");
}
```

Mirror in `tui_app_server/src/chatwidget/tests.rs`.

### Step 7: Add plan-mode scope prompt tests for XHigh

**Audit finding:** The existing plan-scope tests (`plan_reasoning_scope_popup_mentions_selected_reasoning` at line 2838, `plan_reasoning_scope_popup_mentions_built_in_plan_default_when_no_override` at line 2855) only exercise `Medium`/`Low`/`High` — never `XHigh`. Step 3 changes the plan-scope prompt text for XHigh, but without tests those branches ship unverified.

**File:** `codex-rs/tui/src/chatwidget/tests.rs`

Add **four** tests covering both the **selected effort phrase** (line ~7014) and the **override source text** (line ~7025), for both providers:

**Test A — Selected effort phrase, OpenAI:**
```rust
#[tokio::test]
async fn plan_reasoning_scope_popup_shows_extra_high_for_openai() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(Some("gpt-5.1-codex-max")).await;
    chat.open_plan_reasoning_scope_prompt(
        "gpt-5.1-codex-max".to_string(),
        Some(ReasoningEffortConfig::XHigh),
    );
    let popup = render_bottom_popup(&chat, 100);
    assert!(
        popup.contains("extra high reasoning"),
        "OpenAI XHigh should produce 'extra high reasoning' in plan scope; popup: {popup}"
    );
    assert!(
        popup.contains("Always use extra high reasoning in Plan mode."),
        "Plan-only description should use 'extra high'; popup: {popup}"
    );
}
```

**Test B — Selected effort phrase, Claude:**
```rust
#[tokio::test]
async fn plan_reasoning_scope_popup_shows_max_for_claude() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(Some("claude-opus-4-6")).await;
    chat.open_plan_reasoning_scope_prompt(
        "claude-opus-4-6".to_string(),
        Some(ReasoningEffortConfig::XHigh),
    );
    let popup = render_bottom_popup(&chat, 100);
    assert!(
        popup.contains("max reasoning"),
        "Claude XHigh should produce 'max reasoning' in plan scope; popup: {popup}"
    );
    assert!(
        popup.contains("Always use max reasoning in Plan mode."),
        "Plan-only description should use 'max'; popup: {popup}"
    );
}
```

**Test C — Override source text, OpenAI XHigh override:**
Follows the existing pattern from `plan_reasoning_scope_popup_mentions_selected_reasoning` (line 2838) — sets a prior XHigh override, then opens the prompt with a different effort to verify the override source text.
```rust
#[tokio::test]
async fn plan_reasoning_scope_popup_xhigh_override_shows_extra_high_for_openai() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(Some("gpt-5.1-codex-max")).await;
    chat.set_plan_mode_reasoning_effort(Some(ReasoningEffortConfig::XHigh));
    chat.open_plan_reasoning_scope_prompt(
        "gpt-5.1-codex-max".to_string(),
        Some(ReasoningEffortConfig::Medium),
    );
    let popup = render_bottom_popup(&chat, 100);
    assert!(
        popup.contains("user-chosen Plan override (extra high)"),
        "OpenAI XHigh override should show 'extra high' in source text; popup: {popup}"
    );
}
```

**Test D — Override source text, Claude XHigh override:**
```rust
#[tokio::test]
async fn plan_reasoning_scope_popup_xhigh_override_shows_max_for_claude() {
    let (mut chat, _rx, _op_rx) = make_chatwidget_manual(Some("claude-opus-4-6")).await;
    chat.set_plan_mode_reasoning_effort(Some(ReasoningEffortConfig::XHigh));
    chat.open_plan_reasoning_scope_prompt(
        "claude-opus-4-6".to_string(),
        Some(ReasoningEffortConfig::Medium),
    );
    let popup = render_bottom_popup(&chat, 100);
    assert!(
        popup.contains("user-chosen Plan override (max)"),
        "Claude XHigh override should show 'max' in source text; popup: {popup}"
    );
}
```

Mirror all four in `tui_app_server/src/chatwidget/tests.rs`.

### Step 8: Format, lint, test, accept snapshots

```bash
just fmt
just fix -p orbit-code-tui
just fix -p orbit-code-tui-app-server
cargo test -p orbit-code-tui
cargo test -p orbit-code-tui-app-server
cargo insta accept -p orbit-code-tui
cargo insta accept -p orbit-code-tui-app-server
```

No schema regen needed — no protocol/app-server types changed.

## Verification

1. **Build**: `cargo build -p orbit-code-tui -p orbit-code-tui-app-server` passes
2. **Tests**: All tests pass after snapshot acceptance, including:
   - Updated OpenAI popup assertion (Step 5)
   - New Claude popup regression test (Step 6)
   - New plan-scope XHigh selected-effort tests for both providers (Step 7, Tests A+B)
   - New plan-scope XHigh override-source-text tests for both providers (Step 7, Tests C+D)
3. **Lint**: `just fix -p orbit-code-tui` and `just fix -p orbit-code-tui-app-server` pass clean
4. **Visual check — popup**: Run `just codex`, press `/model`, select `gpt-5.4` — verify "Extra High" (not "Max") for the highest level
5. **Visual check — popup**: Select `claude-opus-4-6` — verify "Max" still appears for the highest level
6. **Visual check — warning**: Select `gpt-5.1-codex-max`, navigate to XHigh — warning should say "Extra High reasoning effort can quickly consume Plus plan rate limits."
7. **Visual check — plan scope**: With `gpt-5.4` active and collaboration modes enabled, select XHigh — plan scope prompt should say "extra high reasoning"
8. **Visual check — plan scope**: With `claude-opus-4-6` active, select XHigh — plan scope prompt should say "max reasoning"
9. **Cross-provider switch**: Switch from Claude to OpenAI model via the reasoning popup — label should show the correct provider label even if the selection triggers an auth popup afterward

## Assumptions

- Label casing is `Extra High` (title case, matching existing `Low`/`Medium`/`High` pattern)
- Claude coverage uses `claude-opus-4-6` (current picker-visible Claude preset with xhigh)
- The `warn_for_model` gate only targets GPT slugs, so Claude warning-copy is not affected
- `open_plan_reasoning_scope_prompt` callsites use `.to_lowercase()`, so "Extra High" → "extra high reasoning" reads naturally
- The `claude3` prefix in the heuristic covers legacy Anthropic slug formats; both TUI crates must use the same condition

## Edge Cases Addressed

| Edge Case | Handling |
|-----------|----------|
| Future non-OpenAI, non-Anthropic provider with `XHigh` | Gets "Extra High" (the literal, generic default). Add a new branch when needed. |
| Anthropic slug that doesn't start with `claude-` or `claude3` | Would incorrectly get "Extra High". If Anthropic changes their naming, update the heuristic in both `auth_popup.rs:28` and `tui_app_server/chatwidget.rs`. |
| Heuristic divergence between TUI crates | Mitigated by using the **exact same condition** (`starts_with("claude-") || starts_with("claude3")`). Both locations documented in this plan. |
| Cross-provider switch during reasoning popup | Label is computed from the target model slug, not the current model. Correct label shows regardless of whether auth popup follows. |
| Plan-scope prompt with XHigh (selected effort) | Tested explicitly for both providers — Tests A+B (Step 7). |
| Plan-scope prompt with XHigh (override source text) | Tested explicitly for both providers — Tests C+D (Step 7). Covers the scenario where a prior XHigh override needs provider-correct labeling in the description. |

## Files Changed

| File | Change |
|------|--------|
| `codex-rs/tui/src/chatwidget.rs` | Add `reasoning_effort_label_for_model`, update 5 callsites |
| `codex-rs/tui_app_server/src/chatwidget.rs` | Mirror: add helper with inline heuristic, update 5 callsites |
| `codex-rs/tui/src/chatwidget/tests.rs` | Update OpenAI assertion, add Claude popup test, add 4 plan-scope XHigh tests (selected-effort + override-source, both providers) |
| `codex-rs/tui_app_server/src/chatwidget/tests.rs` | Mirror all test changes (6 new/modified tests total) |
| Snapshot files (auto-generated) | Accept updated OpenAI popup snapshots |

## Not Changed (intentionally)

| File | Why |
|------|-----|
| `codex-rs/protocol/src/openai_models.rs` | No schema changes needed for a UI label fix |
| `codex-rs/core/models.json` | Labels are a TUI concern, not model metadata |
| `codex-rs/app-server-protocol/src/protocol/v2.rs` | No wire format changes |
| `codex-rs/app-server/src/models.rs` | No conversion changes |
| Any `BUILD.bazel` / lockfile | No deps changed |
