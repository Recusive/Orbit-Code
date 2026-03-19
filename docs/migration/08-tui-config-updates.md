# Stage 8: TUI & Config Updates

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a provider/model picker to the TUI, update the onboarding flow for multi-provider auth, add provider status indicators, update the config schema to include new provider settings, and update all help text and branding to reflect multi-provider support.

**Architecture:** The TUI currently hardcodes OpenAI as the provider and shows model selection only for OpenAI models. We extend it with a provider selection step (either during onboarding or via a keybinding), a model picker that lists models from the active provider, and status bar updates showing the active provider and model. Config schema generation is extended to include the new `WireProtocol`, `ProviderConfig`, and `ModelDef` types from `orbit-providers`.

**Tech Stack:** Rust, `ratatui`, `crossterm`, `schemars`.

**Depends on:** Stage 7 (core integration complete -- all providers functional at the engine level).

---

## Files Modified

**TUI crate:**
- `codex-rs/tui/Cargo.toml` -- Add dependency on `orbit-providers`
- `codex-rs/tui/src/lib.rs` -- Add provider selection to startup flow
- `codex-rs/tui/src/cli.rs` -- Add `--provider` CLI argument
- `codex-rs/tui/src/onboarding/auth.rs` -- Multi-provider auth (from Stage 5, polish here)
- `codex-rs/tui/src/onboarding/onboarding_screen.rs` -- Add provider selection step
- `codex-rs/tui/src/oss_selection.rs` -- Extend to support all providers (not just Ollama/LM Studio)
- `codex-rs/tui/src/status/card.rs` -- Show provider + model in session card
- `codex-rs/tui/src/status/account.rs` -- Show provider auth status
- `codex-rs/tui/src/app.rs` -- Add provider/model switch keybinding

**New TUI files:**
- `codex-rs/tui/src/provider_picker.rs` -- Interactive provider/model selection widget

**Config/schema:**
- `codex-rs/core/src/config/schema.rs` -- Include provider types in JSON Schema
- `codex-rs/core/src/config/types.rs` -- Add provider config types to ConfigToml
- `codex-rs/core/src/config/mod.rs` -- Parse provider config from TOML
- `codex-rs/core/config.schema.json` -- Regenerated

**Protocol:**
- `codex-rs/protocol/src/config_types.rs` -- Add provider-related config enums

**Help and branding:**
- `codex-rs/tui/src/version.rs` -- Update version display
- `codex-rs/cli/src/main.rs` -- Update CLI help text

---

### Task 1: Add `--provider` and `--model` CLI Arguments

**Files:**
- Modify: `codex-rs/tui/src/cli.rs`
- Modify: `codex-rs/core/src/config/mod.rs`

- [ ] **Step 1: Add CLI arguments to `Cli` struct**

In `codex-rs/tui/src/cli.rs`, add new arguments:

```rust
/// Provider to use (e.g., "openai", "anthropic", "openrouter", "groq").
/// Overrides the provider from config.toml.
#[arg(long, short = 'P')]
pub provider: Option<String>,

/// Model to use (e.g., "claude-sonnet-4-20250514", "gpt-4.1").
/// Can also use provider/model format (e.g., "anthropic/claude-sonnet-4-20250514").
#[arg(long, short = 'm')]
pub model: Option<String>,
```

- [ ] **Step 2: Parse provider/model compound format**

Support `--model anthropic/claude-sonnet-4-20250514` which sets both provider and model:

```rust
/// Parse a model argument that may contain a provider prefix.
/// Returns (provider_id, model_id).
fn parse_model_arg(model: &str) -> (Option<&str>, &str) {
    if let Some((provider, model)) = model.split_once('/') {
        (Some(provider), model)
    } else {
        (None, model)
    }
}
```

- [ ] **Step 3: Wire into Config**

In `codex-rs/core/src/config/mod.rs`, apply CLI overrides:
```rust
if let Some(provider_id) = &cli.provider {
    config.model_provider = provider_id.clone();
}
if let Some(model_id) = &cli.model {
    let (provider, model) = parse_model_arg(model_id);
    if let Some(p) = provider {
        config.model_provider = p.to_string();
    }
    config.model = model.to_string();
}
```

- [ ] **Step 4: Commit**

```bash
git add codex-rs/tui/src/cli.rs codex-rs/core/src/config/mod.rs
git commit -m "Add --provider and --model CLI arguments with provider/model compound format"
```

---

### Task 2: Create Provider Picker Widget

**Files:**
- Create: `codex-rs/tui/src/provider_picker.rs`

- [ ] **Step 1: Design the provider picker UI**

The picker shows a list of available providers, grouped by category:

```
┌─ Select Provider ──────────────────────────┐
│                                             │
│  Cloud Providers                            │
│  ● OpenAI         ✓ authenticated           │
│  ○ Anthropic      ✓ ANTHROPIC_API_KEY set   │
│  ○ Google Gemini  ✗ needs GOOGLE_API_KEY    │
│  ○ OpenRouter     ✗ needs OPENROUTER_KEY    │
│  ○ Groq           ✓ GROQ_API_KEY set        │
│  ○ Mistral        ✗ needs MISTRAL_API_KEY   │
│  ○ DeepSeek       ✗ needs DEEPSEEK_API_KEY  │
│  ○ xAI            ✗ needs XAI_API_KEY       │
│                                             │
│  Local Providers                            │
│  ○ Ollama         ✓ running on :11434       │
│  ○ LM Studio      ✗ not detected           │
│                                             │
│  Enterprise                                 │
│  ○ Azure OpenAI   ✗ needs configuration     │
│  ○ GitHub Copilot ✗ needs OAuth login       │
│  ○ GitLab         ✗ needs OAuth login       │
│                                             │
│  [Enter] Select  [/] Filter  [Esc] Cancel   │
└─────────────────────────────────────────────┘
```

- [ ] **Step 2: Implement `ProviderPickerWidget`**

```rust
pub struct ProviderPickerWidget {
    providers: Vec<ProviderEntry>,
    selected: usize,
    filter_text: String,
    is_filtering: bool,
}

struct ProviderEntry {
    id: String,
    name: String,
    category: ProviderCategory,
    auth_status: AuthStatus,
}

enum ProviderCategory {
    Cloud,
    Local,
    Enterprise,
}

enum AuthStatus {
    Authenticated,
    EnvVarSet(String),
    NeedsSetup(String),
    Running,       // For local providers
    NotDetected,   // For local providers
}
```

- [ ] **Step 3: Implement keyboard navigation**

- Up/Down arrow: Move selection
- Enter: Select provider and proceed to model selection
- `/`: Enter filter mode (fuzzy search by provider name)
- Esc: Cancel and return to previous screen

- [ ] **Step 4: Implement auth status detection**

For each provider, check:
```rust
fn detect_auth_status(provider: &ProviderConfig) -> AuthStatus {
    // Check env vars
    for env_key in &provider.env_keys {
        if std::env::var(env_key).ok().filter(|v| !v.trim().is_empty()).is_some() {
            return AuthStatus::EnvVarSet(env_key.clone());
        }
    }
    // Check auth.json
    if auth_json.get_provider_auth(&provider.id).is_some() {
        return AuthStatus::Authenticated;
    }
    // Check local provider connectivity
    if provider.auth_type == AuthType::None {
        // Try to connect to the local server
        return check_local_provider_status(provider);
    }
    AuthStatus::NeedsSetup(provider.env_keys.first().cloned().unwrap_or_default())
}
```

- [ ] **Step 5: Commit**

```bash
git add codex-rs/tui/src/provider_picker.rs
git commit -m "Create interactive provider picker widget for TUI"
```

---

### Task 3: Create Model Picker Widget

**Files:**
- Modify: `codex-rs/tui/src/provider_picker.rs` (add model sub-picker)

- [ ] **Step 1: After provider selection, show model picker**

```
┌─ Select Model (Anthropic) ─────────────────┐
│                                             │
│  ● claude-sonnet-4-20250514                 │
│    200K context · $3/$15 per 1M tokens      │
│                                             │
│  ○ claude-opus-4-20250514                   │
│    200K context · $15/$75 per 1M tokens     │
│                                             │
│  ○ claude-haiku-3.5-20241022                │
│    200K context · $0.80/$4 per 1M tokens    │
│                                             │
│  ○ Custom model ID...                       │
│                                             │
│  [Enter] Select  [/] Filter  [Esc] Back     │
└─────────────────────────────────────────────┘
```

- [ ] **Step 2: Show model metadata**

For each model, display:
- Context window (formatted: "200K", "1M")
- Cost per 1M tokens (input/output)
- Capabilities badges: [tools] [images] [reasoning]

- [ ] **Step 3: Allow custom model ID entry**

For providers with dynamic model lists (OpenRouter, Ollama), allow the user to type a custom model ID at the bottom of the list.

- [ ] **Step 4: Fetch models from local providers**

For Ollama and LM Studio, query the provider's `/v1/models` endpoint to list installed models:

```rust
async fn fetch_local_models(provider: &ProviderConfig) -> Vec<String> {
    let url = format!("{}/models", provider.base_url);
    // GET /v1/models returns { data: [{ id: "model-name" }, ...] }
    // Use the existing ollama/lmstudio crate logic
}
```

- [ ] **Step 5: Commit**

```bash
git add codex-rs/tui/src/provider_picker.rs
git commit -m "Add model picker with metadata display and local model fetching"
```

---

### Task 4: Add Provider/Model Switch Keybinding

**Files:**
- Modify: `codex-rs/tui/src/app.rs`

- [ ] **Step 1: Add Ctrl+P keybinding**

In the `App` event handler, add a keybinding to open the provider/model picker:

```rust
// In handle_key_event():
KeyCode::Char('p') if modifiers.contains(KeyModifiers::CONTROL) => {
    self.open_provider_picker();
}
```

- [ ] **Step 2: Handle provider switch during active session**

When the user switches providers mid-session:
1. Store the selection in the config
2. Update the `ModelClient` with the new provider
3. Show a status message: "Switched to Anthropic / claude-sonnet-4-20250514"
4. The next turn uses the new provider

```rust
fn apply_provider_switch(&mut self, provider_id: &str, model_id: &str) {
    // Update runtime config
    self.config.model_provider = provider_id.to_string();
    self.config.model = model_id.to_string();

    // Rebuild the model client for the new provider
    // The agent session will pick up the new client on the next turn
    self.status_message = Some(format!(
        "Switched to {} / {}",
        provider_id, model_id
    ));
}
```

- [ ] **Step 3: Commit**

```bash
git add codex-rs/tui/src/app.rs
git commit -m "Add Ctrl+P keybinding for provider/model switching in TUI"
```

---

### Task 5: Update Status Bar and Session Card

**Files:**
- Modify: `codex-rs/tui/src/status/card.rs`
- Modify: `codex-rs/tui/src/status/account.rs`

- [ ] **Step 1: Show provider in session card**

Currently the session card shows model name. Add the provider:

```
┌─ Session ────────────────────────────────────────────┐
│  Provider: Anthropic                                  │
│  Model: claude-sonnet-4-20250514                      │
│  Context: 200K tokens                                 │
│  Cost: $3.00 / $15.00 per 1M tokens                  │
│  Session: abc123-def456                               │
└──────────────────────────────────────────────────────┘
```

Update the card rendering in `codex-rs/tui/src/status/card.rs` to include:
- Provider name (with wire protocol indicator)
- Model display name
- Context window size
- Cost per 1M tokens (if available)

- [ ] **Step 2: Show auth status in account section**

Update `codex-rs/tui/src/status/account.rs`:
- For API key providers: show "API key: ****XXXX" (last 4 chars)
- For OAuth providers: show "Logged in as {username}"
- For local providers: show "Local (no auth)"

- [ ] **Step 3: Update the header bar**

The TUI header currently shows "Orbit Code". Add the active provider/model to the header:

```
 Orbit Code ─── anthropic/claude-sonnet-4 ─── /path/to/project
```

- [ ] **Step 4: Commit**

```bash
git add codex-rs/tui/src/status/
git commit -m "Update TUI status bar and session card with provider/model info"
```

---

### Task 6: Update Config Schema

**Files:**
- Modify: `codex-rs/core/src/config/schema.rs`
- Modify: `codex-rs/core/src/config/types.rs`
- Modify: `codex-rs/core/src/config/mod.rs`
- Regenerate: `codex-rs/core/config.schema.json`

- [ ] **Step 1: Add provider config types to ConfigToml**

In `codex-rs/core/src/config/types.rs`, add:

```rust
/// Provider-specific configuration in config.toml.
///
/// Example:
/// ```toml
/// [providers.anthropic]
/// env_key = "ANTHROPIC_API_KEY"
///
/// [providers.openrouter]
/// env_key = "OPENROUTER_API_KEY"
/// base_url = "https://openrouter.ai/api/v1"
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct ProviderConfigToml {
    pub base_url: Option<String>,
    pub env_key: Option<String>,
    pub wire_protocol: Option<String>,
    pub http_headers: Option<HashMap<String, String>>,
    pub request_max_retries: Option<u64>,
    pub stream_max_retries: Option<u64>,
    pub stream_idle_timeout_ms: Option<u64>,
}
```

- [ ] **Step 2: Add to ConfigToml**

In the `ConfigToml` struct, add:
```rust
/// Per-provider configuration overrides.
/// Keys are provider IDs (e.g., "anthropic", "openrouter").
pub providers: Option<HashMap<String, ProviderConfigToml>>,

/// Default provider to use when not specified via CLI.
pub default_provider: Option<String>,

/// Default model to use when not specified via CLI.
pub default_model: Option<String>,
```

- [ ] **Step 3: Include WireProtocol in schema**

In `codex-rs/core/src/config/schema.rs`, ensure `WireProtocol` is included in the generated JSON Schema. Since it derives `JsonSchema`, this should work automatically when `ProviderConfigToml` includes it.

- [ ] **Step 4: Regenerate config.schema.json**

```bash
cd codex-rs && cargo run --bin orbit-code-write-config-schema
```

Verify the generated `codex-rs/core/config.schema.json` includes:
- `providers` map with `ProviderConfigToml` schema
- `default_provider` string field
- `default_model` string field
- `wire_protocol` enum with 4 values

- [ ] **Step 5: Update config.md documentation**

Add a section to `codex-rs/config.md` documenting the new provider settings:

```markdown
## Provider Configuration

### Setting a default provider

```toml
default_provider = "anthropic"
default_model = "claude-sonnet-4-20250514"
```

### Provider-specific settings

```toml
[providers.anthropic]
env_key = "ANTHROPIC_API_KEY"

[providers.openrouter]
env_key = "OPENROUTER_API_KEY"
base_url = "https://openrouter.ai/api/v1"

[providers.custom-ollama]
base_url = "http://my-server:11434/v1"
wire_protocol = "chat_completions"
```
```

- [ ] **Step 6: Commit**

```bash
git add codex-rs/core/src/config/ codex-rs/core/config.schema.json codex-rs/config.md
git commit -m "Update config schema with provider settings, default_provider, and default_model"
```

---

### Task 7: Extend OSS Selection for All Providers

**Files:**
- Modify: `codex-rs/tui/src/oss_selection.rs`

- [ ] **Step 1: Extend `SelectOption` for non-OSS providers**

Currently `oss_selection.rs` only shows Ollama and LM Studio. Extend it to also offer API-based providers when no auth is configured:

```rust
// When the user has no OpenAI auth and no local provider running,
// show options for all supported providers
let options = vec![
    SelectOption { label: "OpenAI", key: KeyCode::Char('1'), provider_id: "openai" },
    SelectOption { label: "Anthropic (Claude)", key: KeyCode::Char('2'), provider_id: "anthropic" },
    SelectOption { label: "Google Gemini", key: KeyCode::Char('3'), provider_id: "google" },
    SelectOption { label: "OpenRouter", key: KeyCode::Char('4'), provider_id: "openrouter" },
    SelectOption { label: "Groq", key: KeyCode::Char('5'), provider_id: "groq" },
    SelectOption { label: "Ollama (local)", key: KeyCode::Char('6'), provider_id: "ollama" },
    SelectOption { label: "LM Studio (local)", key: KeyCode::Char('7'), provider_id: "lmstudio" },
];
```

- [ ] **Step 2: After selection, route to appropriate auth flow**

When the user selects a provider:
- If the provider needs an API key, prompt for it (or show env var instructions)
- If the provider uses OAuth, start the device code flow
- If the provider is local, check connectivity and proceed

- [ ] **Step 3: Commit**

```bash
git add codex-rs/tui/src/oss_selection.rs
git commit -m "Extend provider selection to include all supported providers"
```

---

### Task 8: Update Help Text and Branding

**Files:**
- Modify: `codex-rs/cli/src/main.rs` -- Update CLI help text
- Modify: `codex-rs/tui/src/version.rs` -- Update version display
- Modify: `codex-rs/tui/src/onboarding/welcome.rs` -- Update welcome text

- [ ] **Step 1: Update CLI help text**

In the CLI `about` text, mention multi-provider support:

```rust
#[command(about = "Orbit Code — AI-powered terminal coding agent. Supports OpenAI, Anthropic, Google, and 15+ providers.")]
```

- [ ] **Step 2: Update welcome screen**

In `codex-rs/tui/src/onboarding/welcome.rs`, update the welcome text:

```
Welcome to Orbit Code!

Orbit Code is your AI coding assistant in the terminal.
It works with OpenAI, Anthropic (Claude), Google (Gemini),
and many more providers. Press Enter to get started.
```

- [ ] **Step 3: Update tooltips**

In `codex-rs/tui/tooltips.txt`, add tooltips for multi-provider features:
- "Press Ctrl+P to switch provider/model"
- "Configure providers in ~/.orbit-code/config.toml"

- [ ] **Step 4: Commit**

```bash
git add codex-rs/cli/src/main.rs codex-rs/tui/src/version.rs codex-rs/tui/src/onboarding/welcome.rs codex-rs/tui/tooltips.txt
git commit -m "Update help text and branding for multi-provider support"
```

---

### Task 9: Final Verification

- [ ] **Step 1: Full workspace build**

```bash
cd codex-rs && cargo build
```

- [ ] **Step 2: Run all tests**

```bash
cd codex-rs && cargo test
```

- [ ] **Step 3: Manual TUI test**

```bash
cd codex-rs && cargo run --bin orbit-code -- --help
```

Verify:
- `--provider` and `--model` flags appear in help
- Help text mentions multi-provider support

- [ ] **Step 4: Test provider picker rendering**

```bash
# Set an Anthropic key and verify it shows as authenticated
ANTHROPIC_API_KEY=test cargo run --bin orbit-code -- --provider anthropic
```

- [ ] **Step 5: Regenerate config schema and verify**

```bash
cd codex-rs && just write-config-schema
# Check that config.schema.json includes new provider fields
grep -c "providers" core/config.schema.json
grep -c "wire_protocol" core/config.schema.json
grep -c "default_provider" core/config.schema.json
```

- [ ] **Step 6: Commit and push**

```bash
git add -A
git commit -m "Stage 8 complete: TUI provider/model picker, config schema, help text updates"
git push origin main
```

---

## Expected Outcomes

After Stage 8 (final stage):
- Users can select any of 20+ providers via `--provider` CLI flag or interactive picker
- `--model provider/model` compound format works (e.g., `--model anthropic/claude-sonnet-4-20250514`)
- Ctrl+P opens a full-screen provider/model picker with auth status indicators
- Session card shows active provider, model, context window, and cost
- Config schema includes `providers`, `default_provider`, `default_model` settings
- `config.toml` supports per-provider configuration overrides
- Welcome screen and help text reflect multi-provider capability
- OSS selection offers all providers (not just Ollama/LM Studio)
- The entire migration from OpenAI-only fork to multi-provider Orbit Code is complete
- ~1,500 lines of new TUI code + config schema updates
