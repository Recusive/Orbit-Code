# Stage 3c: OpenRouter Provider (Chat Completions API)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add OpenRouter support to Orbit Code so users can access 200+ models (Claude, GPT, Gemini, DeepSeek, Mistral, Llama, etc.) with a single `OPENROUTER_API_KEY`. OpenRouter speaks the standard Chat Completions API (`/v1/chat/completions`), so this also unlocks any other Chat Completions-compatible provider (Groq, Together, DeepSeek direct, Cerebras, etc.) as a config-only addition.

**Architecture:** New `orbit-chat-completions` crate for the Chat Completions wire protocol. Extend `WireApi` enum with `ChatCompletions` variant. Add `openrouter` built-in provider config. No OAuth — API key only.

**Tech Stack:** Rust, `reqwest`, `eventsource-stream`, `serde`/`serde_json`, `tokio`.

**Depends on:** Stage 2 complete (dead crates removed). Can be done in parallel with Stage 3a (Claude provider) — no shared files except `WireApi` enum and `core::client.rs` dispatch.

---

## Agent-Backend Spec References

| Reference | Path (relative to repo root) | What to learn |
|-----------|------|---------------|
| **OpenRouter custom loader** | `reference/opencode/packages/opencode/src/provider/provider.ts:499-509` | Headers: `HTTP-Referer: https://orbit.build/`, `X-Title: orbit` |
| **Chat Completions SDK** | `reference/opencode/packages/opencode/src/provider/provider.ts:183` (BUNDLED_PROVIDERS `@ai-sdk/openai`) | How Chat Completions is instantiated with options |
| **SDK instantiation** | `reference/opencode/packages/opencode/src/provider/provider.ts:1264-1379` | How options (apiKey, headers, custom fetch) are merged |
| **Message transform** | `reference/opencode/packages/opencode/src/provider/transform.ts` | How messages are normalized for Chat Completions format |
| **Provider schema** | `reference/opencode/packages/opencode/src/provider/schema.ts` | ProviderID, ModelID types |
| **Auth service** | `reference/opencode/packages/opencode/src/auth/service.ts` | How API keys are stored/loaded |

---

## Public Surface Changes

| Surface | Before | After |
|---------|--------|-------|
| `orbit-code --model-provider openrouter --model anthropic/claude-sonnet-4-5` | Not supported | **Works** — routes through OpenRouter |
| `config.toml` `model_provider` | `openai`, `ollama`, `lmstudio` | **+ `openrouter`** (+ any Chat Completions-compatible endpoint via custom config) |
| Environment variables | `OPENAI_API_KEY` only | **+ `OPENROUTER_API_KEY`** |

---

## Files Created

**New crate: `orbit-chat-completions`** (Chat Completions API SSE client)
- `codex-rs/orbit-chat-completions/Cargo.toml`
- `codex-rs/orbit-chat-completions/src/lib.rs`
- `codex-rs/orbit-chat-completions/src/types.rs` — Request/response serde types
- `codex-rs/orbit-chat-completions/src/client.rs` — `ChatCompletionsClient` with SSE streaming
- `codex-rs/orbit-chat-completions/src/stream.rs` — SSE event parsing (`data:` JSON lines, `[DONE]` sentinel)
- `codex-rs/orbit-chat-completions/src/error.rs` — Error types

## Files Modified

**Workspace:**
- `codex-rs/Cargo.toml` — add workspace member + dependency

**Core (wire dispatch + provider config):**
- `codex-rs/core/src/model_provider_info.rs` — add `ChatCompletions` to `WireApi`, add `openrouter` built-in provider
- `codex-rs/core/src/client.rs` — dispatch to Chat Completions client when `wire_api == ChatCompletions`

---

### Task 1: Create `orbit-chat-completions` Crate (Wire Protocol)

Implements the OpenAI Chat Completions API (`POST /v1/chat/completions`) with SSE streaming. This is the standard protocol used by OpenRouter, Groq, Mistral, DeepSeek, Together, Cerebras, DeepInfra, Azure, Ollama (legacy mode), and any OpenAI-compatible endpoint.

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "orbit-chat-completions"
version.workspace = true
edition.workspace = true
license.workspace = true

[lints]
workspace = true

[dependencies]
orbit-code-client = { workspace = true }
anyhow = { workspace = true }
eventsource-stream = { workspace = true }
futures = { workspace = true }
http = { workspace = true }
reqwest = { workspace = true }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
thiserror = { workspace = true }
tokio = { workspace = true, features = ["sync"] }
tracing = { workspace = true }

[dev-dependencies]
pretty_assertions = { workspace = true }
tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }
```

- [ ] **Step 2: Define request/response types in `types.rs`**

Match the OpenAI Chat Completions API spec:

```rust
// --- Request ---

#[derive(Debug, Serialize)]
pub struct ChatCompletionsRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: ChatContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

// Content can be a string or array of content parts (for images)
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ChatContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Serialize)]
pub struct StreamOptions {
    pub include_usage: bool,
}

// --- Response (streaming) ---

#[derive(Debug, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub choices: Vec<ChunkChoice>,
    #[serde(default)]
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
pub struct ChunkChoice {
    pub index: u32,
    pub delta: ChunkDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChunkDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ChunkToolCall>>,
    /// Extended thinking (DeepSeek, QwQ)
    pub reasoning_content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChunkToolCall {
    pub index: u32,
    pub id: Option<String>,
    pub function: Option<ChunkFunction>,
}

#[derive(Debug, Deserialize)]
pub struct ChunkFunction {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}
```

- [ ] **Step 3: Implement SSE parsing in `stream.rs`**

Chat Completions SSE is simpler than Anthropic — each `data:` line is a JSON `ChatCompletionChunk`. The stream ends with `data: [DONE]`.

```rust
pub enum ChatCompletionEvent {
    Delta {
        content: Option<String>,
        tool_calls: Option<Vec<ChunkToolCall>>,
        reasoning: Option<String>,
        finish_reason: Option<String>,
    },
    Usage(Usage),
    Done,
    Error(String),
}
```

Parse loop:
1. Read SSE `data:` line
2. If `data: [DONE]` → emit `Done`
3. Parse JSON as `ChatCompletionChunk`
4. For each choice, emit `Delta` with content/tool_calls/reasoning
5. If chunk has `usage`, emit `Usage`

- [ ] **Step 4: Implement `ChatCompletionsClient` in `client.rs`**

```rust
pub struct ChatCompletionsClient {
    transport: Arc<dyn HttpTransport>,
    base_url: String,
}

impl ChatCompletionsClient {
    pub async fn stream(
        &self,
        request: ChatCompletionsRequest,
        api_key: &str,
        extra_headers: HeaderMap,
    ) -> Result<ChatCompletionStream, ChatCompletionsError>
}
```

Request construction:
- `POST {base_url}/chat/completions`
- Headers: `Authorization: Bearer {api_key}`, `Content-Type: application/json`, `Accept: text/event-stream`
- Body: serialized `ChatCompletionsRequest` with `stream: true`, `stream_options: { include_usage: true }`
- Extra headers merged (for provider-specific headers like OpenRouter's `HTTP-Referer`)

Uses `orbit_code_client` transport for the HTTP request and SSE byte stream.

- [ ] **Step 5: Define error types in `error.rs`**

```rust
#[derive(Debug, Error)]
pub enum ChatCompletionsError {
    #[error(transparent)]
    Transport(#[from] orbit_code_client::TransportError),
    #[error("Chat Completions API error ({status}): {message}")]
    Api { status: u16, message: String, code: Option<String> },
    #[error("SSE parse error: {0}")]
    StreamParse(String),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
```

- [ ] **Step 6: Wire up `lib.rs`**

```rust
pub mod client;
pub mod error;
pub mod stream;
pub mod types;

pub use client::ChatCompletionsClient;
pub use error::ChatCompletionsError;
pub use stream::ChatCompletionEvent;
pub use stream::ChatCompletionStream;
pub use types::*;
```

- [ ] **Step 7: Add to workspace**

In `codex-rs/Cargo.toml`, add to members and workspace.dependencies:
```toml
# members
"orbit-chat-completions",
# workspace.dependencies
orbit-chat-completions = { path = "orbit-chat-completions" }
```

- [ ] **Step 8: cargo check + commit**

```bash
cd codex-rs && cargo check -p orbit-chat-completions
git add codex-rs/orbit-chat-completions/ codex-rs/Cargo.toml codex-rs/Cargo.lock
git commit -m "Add orbit-chat-completions crate: Chat Completions API SSE client"
```

---

### Task 2: Wire OpenRouter into Core (Provider Config + Dispatch)

- [ ] **Step 1: Extend `WireApi` enum**

In `codex-rs/core/src/model_provider_info.rs`:

```rust
pub enum WireApi {
    #[default]
    Responses,
    /// Chat Completions API at `/v1/chat/completions` (OpenRouter, Groq, etc.)
    ChatCompletions,
}
```

(If Stage 3a has already added `AnthropicMessages`, just add `ChatCompletions` alongside it.)

- [ ] **Step 2: Add `openrouter` built-in provider**

**Spec reference:** `reference/opencode/packages/opencode/src/provider/provider.ts:499-509`

```rust
ModelProviderInfo {
    name: "OpenRouter".to_string(),
    base_url: Some("https://openrouter.ai/api/v1".to_string()),
    env_key: Some("OPENROUTER_API_KEY".to_string()),
    env_key_instructions: Some("Get your API key at https://openrouter.ai/keys".to_string()),
    wire_api: WireApi::ChatCompletions,
    http_headers: Some(HashMap::from([
        ("HTTP-Referer".to_string(), "https://orbit.build/".to_string()),
        ("X-Title".to_string(), "orbit".to_string()),
    ])),
    requires_openai_auth: false,
    supports_websockets: false,
    ..Default::default()
}
```

- [ ] **Step 3: Add dispatch in `core::client.rs`**

In the request flow, branch on `wire_api`:

```rust
match provider.wire_api {
    WireApi::Responses => {
        // existing ResponsesClient path
    }
    WireApi::ChatCompletions => {
        // new: create ChatCompletionsClient
        // translate internal request → ChatCompletionsRequest
        // translate internal tools → Chat Completions tool format
        // stream via Chat Completions SSE
        // map ChatCompletionEvent → internal ResponseEvent format
    }
}
```

Key translation work:
- Internal tool definitions → Chat Completions `tools` array with `function` type
- Internal message format → `ChatMessage` with role/content/tool_calls
- `ChatCompletionEvent::Delta` → internal text/tool-call events
- `ChatCompletionEvent::Usage` → internal token usage tracking
- `ChunkToolCall` accumulation — tool call arguments arrive as incremental string fragments across multiple deltas; accumulate by `index` until `finish_reason: "tool_calls"`

- [ ] **Step 4: Commit**

```bash
cd codex-rs && cargo check
git add codex-rs/core/
git commit -m "Wire OpenRouter provider: WireApi::ChatCompletions, built-in config, dispatch"
```

---

### Task 3: Add Other Chat Completions Providers (Config Only)

Since the `orbit-chat-completions` crate handles the wire protocol generically, adding more providers is just config:

- [ ] **Step 1: Document user-configurable providers**

Users can add any Chat Completions-compatible provider in `config.toml`:

```toml
[model_providers.groq]
name = "Groq"
base_url = "https://api.groq.com/openai/v1"
env_key = "GROQ_API_KEY"
wire_api = "chatcompletions"

[model_providers.deepseek]
name = "DeepSeek"
base_url = "https://api.deepseek.com/v1"
env_key = "DEEPSEEK_API_KEY"
wire_api = "chatcompletions"

[model_providers.together]
name = "Together AI"
base_url = "https://api.together.xyz/v1"
env_key = "TOGETHER_API_KEY"
wire_api = "chatcompletions"

[model_providers.mistral]
name = "Mistral"
base_url = "https://api.mistral.ai/v1"
env_key = "MISTRAL_API_KEY"
wire_api = "chatcompletions"
```

No code changes needed — the existing `ModelProviderInfo` config structure + `WireApi::ChatCompletions` handles all of these.

- [ ] **Step 2: Update docs**

Update `docs/codex/config.md` with:
- OpenRouter setup instructions
- How to add custom Chat Completions providers
- List of tested providers (OpenRouter, Groq, DeepSeek, Together, Mistral)

- [ ] **Step 3: Commit**

```bash
git add docs/
git commit -m "Document OpenRouter and custom Chat Completions provider setup"
```

---

### Task 4: Add Unit Tests

- [ ] **Step 1: SSE parsing tests**

Test the Chat Completions SSE format:
- Parse text content deltas
- Parse tool call deltas (incremental argument fragments across multiple chunks)
- Parse `[DONE]` sentinel
- Parse usage from final chunk
- Parse `reasoning_content` for DeepSeek/QwQ models
- Handle empty delta (heartbeat)
- Handle error responses (non-200 status before SSE starts)

- [ ] **Step 2: Tool call accumulation tests**

Tool call arguments arrive fragmented:
```
data: {"choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_1","function":{"name":"shell","arguments":""}}]}}]}
data: {"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"{\"co"}}]}}]}
data: {"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"mmand"}}]}}]}
data: {"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"\":\"ls\"}"}}]}}]}
```

Test that the accumulator correctly assembles `{"command":"ls"}` from fragments.

- [ ] **Step 3: Request construction tests**

Verify:
- `Authorization: Bearer {key}` header is set
- Extra headers (OpenRouter's `HTTP-Referer`, `X-Title`) are merged
- `stream: true` and `stream_options.include_usage: true` are in the body
- Tool definitions are serialized correctly

- [ ] **Step 4: Run tests + commit**

```bash
cd codex-rs && cargo test -p orbit-chat-completions
git add codex-rs/orbit-chat-completions/
git commit -m "Add unit tests for Chat Completions SSE parsing and tool call accumulation"
```

---

### Task 5: Verify and Finalize

- [ ] **Step 1: Full workspace check**

```bash
cd codex-rs && cargo check
```

- [ ] **Step 2: Run affected crate tests**

```bash
cd codex-rs && cargo test -p orbit-chat-completions
cd codex-rs && cargo test -p orbit-code-core
cd codex-rs && cargo test -p orbit-code-cli
```

- [ ] **Step 3: Lockfile maintenance**

```bash
cd codex-rs && cargo generate-lockfile
just bazel-lock-update
just bazel-lock-check
```

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "Stage 3c complete: OpenRouter provider with Chat Completions API"
```

---

## Expected Outcomes

After Stage 3c:
- Users can run `orbit-code --model-provider openrouter --model anthropic/claude-sonnet-4-5` to use Claude through OpenRouter
- Users can set `OPENROUTER_API_KEY` and access 200+ models with one key
- Any Chat Completions-compatible provider can be added via `config.toml` without code changes
- The `orbit-chat-completions` crate is reusable for Groq, DeepSeek, Together, Mistral, etc.
- Tool calling works with Chat Completions format (incremental argument accumulation)
- Streaming text and reasoning content are parsed correctly
- The existing OpenAI Responses API path is unchanged

## Bonus: Providers Unlocked for Free

Once `orbit-chat-completions` exists, these providers work with config-only setup (no code changes):

| Provider | Base URL | Env Key |
|----------|----------|---------|
| Groq | `https://api.groq.com/openai/v1` | `GROQ_API_KEY` |
| DeepSeek | `https://api.deepseek.com/v1` | `DEEPSEEK_API_KEY` |
| Together AI | `https://api.together.xyz/v1` | `TOGETHER_API_KEY` |
| Mistral | `https://api.mistral.ai/v1` | `MISTRAL_API_KEY` |
| Cerebras | `https://api.cerebras.ai/v1` | `CEREBRAS_API_KEY` |
| xAI (Grok) | `https://api.x.ai/v1` | `XAI_API_KEY` |
| Perplexity | `https://api.perplexity.ai` | `PERPLEXITY_API_KEY` |
| DeepInfra | `https://api.deepinfra.com/v1/openai` | `DEEPINFRA_API_KEY` |
