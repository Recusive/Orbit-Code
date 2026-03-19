# Stage 4: Provider Registry

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create an `orbit-providers` crate containing a static registry of 20+ provider definitions with model metadata, capabilities, costs, and context limits -- ported from the TypeScript `provider.ts` and `models.ts` into Rust.

**Architecture:** The crate defines a `ProviderConfig` struct (provider-level settings like base URL, env var for API key, auth type) and a `ModelDef` struct (model-level settings like context window, cost, capabilities). A `WireProtocol` enum (`Responses`, `ChatCompletions`, `Anthropic`, `Gemini`) determines which Stage 3 client crate to use. The registry is a static `HashMap<&str, ProviderConfig>` compiled into the binary with an optional runtime path for fetching models from an API (e.g., `/v1/models`).

**Tech Stack:** Rust, `serde`, `schemars` (for config schema generation).

**Depends on:** Stage 3 (client crates exist, `WireProtocol` variants correspond to real implementations).

---

## Reference: TypeScript Provider Definitions

The TypeScript Agent-backend defines providers in `provider/provider.ts` via `BUNDLED_PROVIDERS` and `CUSTOM_LOADERS`, with model definitions coming from `provider/models.ts` (which fetches from models.dev). Each provider has:

- `id` -- unique string identifier (e.g., `"anthropic"`, `"openrouter"`, `"groq"`)
- `npm` -- which AI SDK package to use (determines wire protocol)
- `env` -- environment variable(s) for the API key
- `url` -- base URL (may contain `${VAR}` template vars)
- `models` -- map of model ID to model definition
- `options` -- provider-specific SDK options
- Custom loaders for special auth (Bedrock, Copilot, Azure)

---

## Files Created

**New crate: `orbit-providers`**
- `codex-rs/orbit-providers/Cargo.toml`
- `codex-rs/orbit-providers/src/lib.rs`
- `codex-rs/orbit-providers/src/wire_protocol.rs` -- `WireProtocol` enum
- `codex-rs/orbit-providers/src/provider_config.rs` -- `ProviderConfig` struct
- `codex-rs/orbit-providers/src/model_def.rs` -- `ModelDef` struct with capabilities and costs
- `codex-rs/orbit-providers/src/registry.rs` -- Static built-in provider registry
- `codex-rs/orbit-providers/src/capabilities.rs` -- Model capability flags
- `codex-rs/orbit-providers/src/cost.rs` -- Token cost definitions

**Modified:**
- `codex-rs/Cargo.toml` -- Add workspace member and dependency
- `codex-rs/core/src/model_provider_info.rs` -- Extend `WireApi` enum (or replace with `WireProtocol` import)

---

### Task 1: Define `WireProtocol` Enum

**Files:**
- Create: `codex-rs/orbit-providers/src/wire_protocol.rs`

- [ ] **Step 1: Define the enum**

```rust
/// Wire protocol that a provider speaks. Determines which client crate
/// handles the HTTP/SSE communication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WireProtocol {
    /// OpenAI Responses API (`/v1/responses`) -- handled by orbit-code-api
    Responses,
    /// OpenAI Chat Completions API (`/v1/chat/completions`) -- handled by orbit-chat-completions
    ChatCompletions,
    /// Anthropic Messages API (`/v1/messages`) -- handled by orbit-anthropic
    Anthropic,
    /// Google GenerateContent API -- handled by orbit-gemini
    Gemini,
}
```

- [ ] **Step 2: Implement Display and Default**

Default should be `Responses` for backward compatibility with existing OpenAI-only config.

- [ ] **Step 3: Add deserialization compatibility**

Accept `"responses"`, `"chat_completions"`, `"anthropic"`, `"gemini"` in TOML config. Reject the legacy `"chat"` value with a helpful error message (matching the existing `CHAT_WIRE_API_REMOVED_ERROR` pattern in `model_provider_info.rs`).

---

### Task 2: Define `ProviderConfig` Struct

**Files:**
- Create: `codex-rs/orbit-providers/src/provider_config.rs`

- [ ] **Step 1: Define provider-level configuration**

```rust
/// Configuration for a model provider (API endpoint).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct ProviderConfig {
    /// Unique provider identifier (e.g., "anthropic", "openrouter").
    pub id: String,

    /// Human-readable display name.
    pub name: String,

    /// Base URL for API requests.
    pub base_url: String,

    /// Environment variable(s) that hold the API key.
    /// First found is used. Empty means no key needed (local providers).
    pub env_keys: Vec<String>,

    /// Instructions for the user on how to obtain/set the API key.
    pub env_key_instructions: Option<String>,

    /// Wire protocol this provider speaks.
    pub wire_protocol: WireProtocol,

    /// Authentication type for this provider.
    pub auth_type: AuthType,

    /// Whether this provider requires OpenAI-style OAuth login.
    pub requires_openai_auth: bool,

    /// Whether this provider supports WebSocket transport (Responses API only).
    pub supports_websockets: bool,

    /// Optional query parameters appended to every request URL.
    pub query_params: Option<HashMap<String, String>>,

    /// Static HTTP headers included in every request.
    pub http_headers: Option<HashMap<String, String>>,

    /// HTTP headers whose values come from environment variables.
    pub env_http_headers: Option<HashMap<String, String>>,

    /// Retry configuration overrides.
    pub request_max_retries: Option<u64>,
    pub stream_max_retries: Option<u64>,
    pub stream_idle_timeout_ms: Option<u64>,

    /// Built-in model definitions for this provider.
    pub models: HashMap<String, ModelDef>,
}

/// How the provider authenticates requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    /// API key in Authorization: Bearer header (most providers).
    BearerToken,
    /// API key in x-api-key header (Anthropic).
    ApiKeyHeader,
    /// API key as query parameter (Gemini).
    QueryParam,
    /// OAuth bearer token (GitHub Copilot, GitLab).
    OAuth,
    /// AWS IAM credentials (Bedrock).
    AwsIam,
    /// No authentication needed (local providers like Ollama, LM Studio).
    None,
}
```

---

### Task 3: Define `ModelDef` Struct

**Files:**
- Create: `codex-rs/orbit-providers/src/model_def.rs`
- Create: `codex-rs/orbit-providers/src/capabilities.rs`
- Create: `codex-rs/orbit-providers/src/cost.rs`

- [ ] **Step 1: Define model capabilities**

Port from the TypeScript `ModelsDev.Model` type which tracks input/output modality support:

```rust
/// What a model can accept and produce.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ModelCapabilities {
    /// Input modalities this model accepts.
    pub input: InputCapabilities,
    /// Output modalities this model produces.
    pub output: OutputCapabilities,
    /// Whether this model supports extended thinking / reasoning.
    pub reasoning: bool,
    /// Whether this model supports tool/function calling.
    pub tool_use: bool,
    /// Whether this model supports streaming.
    pub streaming: bool,
    /// Supported reasoning effort levels (e.g., ["low", "medium", "high"]).
    pub reasoning_efforts: Vec<String>,
    /// Whether interleaved thinking is supported, and if so, the field name.
    pub interleaved: Option<InterleavedConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct InputCapabilities {
    pub text: bool,
    pub image: bool,
    pub audio: bool,
    pub video: bool,
    pub pdf: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct OutputCapabilities {
    pub text: bool,
    pub image: bool,
    pub audio: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InterleavedConfig {
    /// JSON field name for reasoning content (e.g., "reasoning_content", "reasoning_details").
    pub field: String,
}
```

- [ ] **Step 2: Define model cost structure**

```rust
/// Per-token cost in USD.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ModelCost {
    /// Cost per input token in USD.
    pub input: f64,
    /// Cost per output token in USD.
    pub output: f64,
    /// Cost per cached input token (if provider supports caching).
    pub cache_read: Option<f64>,
    /// Cost to create cache entry per token.
    pub cache_write: Option<f64>,
}
```

- [ ] **Step 3: Define the full model definition**

```rust
/// Complete definition for a model available through a provider.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModelDef {
    /// Model identifier as sent to the API (e.g., "claude-sonnet-4-20250514").
    pub api_id: String,

    /// Human-readable display name (e.g., "Claude Sonnet 4").
    pub display_name: String,

    /// Maximum context window in tokens.
    pub context_window: u64,

    /// Default max output tokens (provider may cap lower).
    pub default_max_output_tokens: u64,

    /// Maximum possible output tokens.
    pub max_output_tokens: u64,

    /// Per-token pricing.
    pub cost: ModelCost,

    /// Model capabilities.
    pub capabilities: ModelCapabilities,

    /// Provider-specific temperature override (Some providers need specific defaults).
    /// Reference: ProviderTransform.temperature() in transform.ts
    pub default_temperature: Option<f64>,

    /// Provider-specific top_p override.
    pub default_top_p: Option<f64>,

    /// Provider-specific top_k override.
    pub default_top_k: Option<u64>,
}
```

---

### Task 4: Build the Static Provider Registry

**Files:**
- Create: `codex-rs/orbit-providers/src/registry.rs`

- [ ] **Step 1: Define the built-in provider list**

Port the 20+ providers from the TypeScript `BUNDLED_PROVIDERS` map and `CUSTOM_LOADERS`. Each entry maps a provider ID to a `ProviderConfig`. Key providers to include:

| Provider ID | Wire Protocol | Base URL | Env Key(s) |
|------------|--------------|----------|------------|
| `openai` | Responses | `https://api.openai.com/v1` | `OPENAI_API_KEY` |
| `anthropic` | Anthropic | `https://api.anthropic.com` | `ANTHROPIC_API_KEY` |
| `google` | Gemini | `https://generativelanguage.googleapis.com` | `GOOGLE_GENERATIVE_AI_API_KEY`, `GOOGLE_API_KEY` |
| `google-vertex` | Gemini | `https://${GOOGLE_VERTEX_LOCATION}-aiplatform.googleapis.com/v1` | (OAuth) |
| `openrouter` | ChatCompletions | `https://openrouter.ai/api/v1` | `OPENROUTER_API_KEY` |
| `groq` | ChatCompletions | `https://api.groq.com/openai/v1` | `GROQ_API_KEY` |
| `mistral` | ChatCompletions | `https://api.mistral.ai/v1` | `MISTRAL_API_KEY` |
| `deepseek` | ChatCompletions | `https://api.deepseek.com/v1` | `DEEPSEEK_API_KEY` |
| `xai` | ChatCompletions | `https://api.x.ai/v1` | `XAI_API_KEY` |
| `together` | ChatCompletions | `https://api.together.xyz/v1` | `TOGETHER_AI_API_KEY` |
| `perplexity` | ChatCompletions | `https://api.perplexity.ai` | `PERPLEXITY_API_KEY` |
| `cerebras` | ChatCompletions | `https://api.cerebras.ai/v1` | `CEREBRAS_API_KEY` |
| `deepinfra` | ChatCompletions | `https://api.deepinfra.com/v1/openai` | `DEEPINFRA_API_KEY` |
| `azure` | ChatCompletions | `https://${AZURE_RESOURCE_NAME}.openai.azure.com/openai` | `AZURE_API_KEY` |
| `ollama` | ChatCompletions | `http://localhost:11434/v1` | (none) |
| `lmstudio` | ChatCompletions | `http://localhost:1234/v1` | (none) |
| `github-copilot` | ChatCompletions | (dynamic) | (OAuth) |
| `gitlab` | ChatCompletions | (dynamic) | (OAuth) |
| `cohere` | ChatCompletions | `https://api.cohere.com/v2` | `COHERE_API_KEY` |
| `alibaba` | ChatCompletions | `https://dashscope.aliyuncs.com/compatible-mode/v1` | `DASHSCOPE_API_KEY` |
| `glm` | ChatCompletions | `https://open.bigmodel.cn/api/paas/v4` | `GLM_API_KEY` |

```rust
pub fn built_in_providers() -> HashMap<String, ProviderConfig> {
    let mut providers = HashMap::new();
    // ... register each provider
    providers
}
```

- [ ] **Step 2: Populate model definitions for key providers**

For the initial release, include at least the flagship models for each provider:

**Anthropic:**
- `claude-sonnet-4-20250514` -- 200K context, $3/$15 per 1M tokens
- `claude-opus-4-20250514` -- 200K context, $15/$75 per 1M tokens
- `claude-haiku-3.5-20241022` -- 200K context, $0.80/$4 per 1M tokens

**OpenAI (keep existing, add Chat Completions variants):**
- `gpt-4.1` -- 1M context
- `gpt-4o` -- 128K context
- `o3-mini` -- 200K context

**Google Gemini:**
- `gemini-2.5-pro` -- 1M context
- `gemini-2.5-flash` -- 1M context

**OpenRouter, Groq, etc.:** Start with empty model lists -- users select models by ID; the registry provides the connection info. Optionally add top models.

- [ ] **Step 3: Add `get_provider` and `find_model` lookup functions**

```rust
/// Look up a provider by ID, checking user config first, then built-in registry.
pub fn get_provider(id: &str) -> Option<&ProviderConfig> { ... }

/// Find which provider owns a given model ID.
pub fn find_model(model_id: &str) -> Option<(&ProviderConfig, &ModelDef)> { ... }

/// List all available providers (built-in + user-defined).
pub fn all_providers() -> Vec<&ProviderConfig> { ... }
```

- [ ] **Step 4: Add runtime model fetching stub**

```rust
/// Fetch available models from a provider's API (e.g., GET /v1/models).
/// This is used for providers like Ollama and LM Studio where models
/// are installed locally and not known at compile time.
pub async fn fetch_models(provider: &ProviderConfig) -> Result<Vec<String>, ProviderError> {
    // For now, stub this out. The existing ollama and lmstudio crates
    // already implement model fetching -- we'll wire them in during Stage 7.
    Ok(vec![])
}
```

- [ ] **Step 5: Commit**

```bash
git add codex-rs/orbit-providers/
git commit -m "Add orbit-providers crate: static provider registry with 20+ providers and model definitions"
```

---

### Task 5: Register Crate in Workspace and Verify

**Files:**
- Modify: `codex-rs/Cargo.toml`

- [ ] **Step 1: Add to workspace**

Add to members:
```toml
"orbit-providers",
```

Add to workspace.dependencies:
```toml
orbit-providers = { path = "orbit-providers" }
```

- [ ] **Step 2: cargo check**

```bash
cd codex-rs && cargo check -p orbit-providers
```

- [ ] **Step 3: Run tests**

```bash
cd codex-rs && cargo test -p orbit-providers
```

- [ ] **Step 4: Commit and push**

```bash
git add -A
git commit -m "Stage 4 complete: provider registry with 20+ providers, model definitions, WireProtocol enum"
git push origin main
```

---

## Expected Outcomes

After Stage 4:
- `orbit-providers` crate exists with a complete registry of 20+ providers
- Each provider has: ID, name, base URL, env key, wire protocol, auth type, and model definitions
- `WireProtocol` enum with 4 variants maps to the Stage 3 client crates
- `ModelDef` includes context windows, costs, and capability flags
- Static registry is compiled into the binary for zero-config startup
- Runtime model fetching is stubbed for local providers (Ollama, LM Studio)
- Config schema generation can include the new provider types
- ~1,500 lines of new code
