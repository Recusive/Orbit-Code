# Stage 3a: Claude Provider (Anthropic Messages API — API Key Auth)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Claude model support to Orbit Code with **API key auth only** (`ANTHROPIC_API_KEY` env var). In stage `3a`, users must set the env var, specify `model_provider = "anthropic"`, and select an explicit Claude model via config `model` or `--model`.

**Scope reduction rationale (from audit):** The original plan attempted OAuth, persisted auth, login CLI/TUI/app-server integration, and automatic model-based provider routing — all in one stage. Audit found that auth storage (`AuthDotJson`), login flows (`login.rs`, `server.rs`), TUI onboarding, app-server protocol (`AuthMode`, `LoginAccountParams`), and provider routing are all deeply coupled to OpenAI/ChatGPT. Trying to generalize all of them simultaneously is high-risk. This revised plan keeps stage 3a to the **minimum viable provider**: wire protocol + API key via env var + explicit provider selection.

**What's deferred to stage 3b:**
- Anthropic OAuth (code-paste flow) — requires login crate redesign, new app-server protocol states, TUI onboarding rework
- Provider-scoped auth storage — requires `AuthDotJson` migration to multi-provider format
- Persisted Anthropic API key in `auth.json` — requires auth storage redesign
- Automatic model→provider routing and provider-aware picker filtering for Claude presets
- Shared model metadata extensions for Anthropic-only defaults such as `max_tokens`, thinking mode, and 1M-context beta requirements

**Architecture:** New `orbit-code-anthropic` crate for the Anthropic Messages API wire protocol. Extend `WireApi` enum with `AnthropicMessages` variant, serialized as `wire_api = "anthropic_messages"`. Add `anthropic` built-in provider to `model_provider_info.rs` with `env_key = "ANTHROPIC_API_KEY"`. Wire dispatch in `core::client.rs`, keep Anthropic request defaults in code keyed by model slug, and add hidden Claude bundled entries so manual `--model claude-*` works with `model_provider = "anthropic"` without exposing Claude presets in pickers before provider-aware filtering exists. No changes to auth storage, login flows, TUI, app-server protocol, or shared `AuthMode` enum.

**Tech Stack:** Rust, `reqwest`, `eventsource-stream`, `serde`/`serde_json`, `tokio`.

**Depends on:** Stage 2 complete (dead crates removed).

---

## Local Stage 3a Protocol Notes

This document is the local implementation reference for stage 3a. Do not require external `reference/opencode...` files to complete this stage.

- **Headers:** Always send `anthropic-version: 2023-06-01`, `Content-Type: application/json`, and `anthropic-beta: claude-code-20250219,interleaved-thinking-2025-05-14,fine-grained-tool-streaming-2025-05-14`. Append `context-1m-2025-08-07` for `claude-sonnet-4-6` and `claude-opus-4-6`. No OAuth-only headers or `?beta=true` query parameter in 3a.
- **Request shaping:** Filter empty messages and empty text blocks that Anthropic rejects, sanitize tool call IDs to `[a-zA-Z0-9_-]`, and omit `temperature` so Anthropic uses its default.
- **Explicit model requirement:** Stage `3a` does not support implicit default-model selection for Anthropic. If `model_provider = "anthropic"` is set and no explicit Claude model is selected, fail fast with `Anthropic stage 3a requires an explicit Claude model when \`model_provider = "anthropic"\` is set.`
- **Per-model defaults:** Keep Anthropic-specific `max_tokens`, thinking mode, and 1M-context handling in code keyed by model slug. Do not extend `models.json` with Anthropic-only metadata in 3a.
- **Supported tool subset:** Support only `ToolSpec::Function` in 3a. Reject `ToolSpec::ToolSearch`, `ToolSpec::LocalShell`, `ToolSpec::ImageGeneration`, `ToolSpec::WebSearch`, and `ToolSpec::Freeform` with a clear invalid-request error for Anthropic.
- **Streaming events:** Parse `message_start`, `content_block_start`, `content_block_delta`, `content_block_stop`, `message_delta`, `message_stop`, `ping`, and `error`. Map thinking deltas to `ResponseEvent::ReasoningContentDelta`; do not describe Anthropic thinking as `ReasoningSummaryDelta` in 3a.


---

## Public Surface Changes


| Surface | Before | After (3a) |
| ------- | ------ | ---------- |
| `config.toml` `model_provider` | `openai`, `ollama`, `lmstudio` | **+ `anthropic`** |
| `ANTHROPIC_API_KEY` env var | Ignored | **Used when `model_provider = "anthropic"`** |
| Anthropic model selection | Not supported | **Explicit Claude `model` / `--model` required in stage `3a`** |
| `model_provider = "anthropic"` with no model | Not supported | **Fails fast with a configuration error; no Anthropic default model in stage `3a`** |
| `orbit-code --model claude-sonnet-4-6` | Not supported | **Works when `model_provider = "anthropic"` is set** |
| Model picker | Provider-visible models only | **Claude bundled presets stay hidden in 3a; manual model entry only** |
| Login flow | Unchanged | **Unchanged** (no new login paths in 3a) |
| Auth storage | Unchanged | **Unchanged** (no `auth.json` changes in 3a) |
| App-server protocol | Unchanged | **Unchanged** (no `AuthMode`/`LoginAccountParams` changes in 3a) |
| TUI onboarding | Unchanged | **Unchanged** (no new auth UI in 3a) |

**Usage for stage 3a:**
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
echo 'model_provider = "anthropic"' >> ~/.codex/config.toml
echo 'model = "claude-sonnet-4-6"' >> ~/.codex/config.toml
orbit-code

# Or provide the Claude model explicitly on the command line
orbit-code --model claude-sonnet-4-6
```

If `model_provider = "anthropic"` is set without an explicit Claude model, fail fast with:

```text
Anthropic stage 3a requires an explicit Claude model when `model_provider = "anthropic"` is set.
```


---

## Files Created

**New crate: `orbit-code-anthropic`** (Anthropic Messages API SSE client)

Following existing provider-crate naming convention (matches `orbit-code-ollama`, `orbit-code-lmstudio`):

- `codex-rs/anthropic/Cargo.toml` — package name `orbit-code-anthropic`
- `codex-rs/anthropic/BUILD.bazel` — Bazel target (use `codex-rs/ollama/BUILD.bazel` as template)
- `codex-rs/anthropic/src/lib.rs`
- `codex-rs/anthropic/src/types.rs` — Request/response serde types
- `codex-rs/anthropic/src/client.rs` — `AnthropicClient` with SSE streaming
- `codex-rs/anthropic/src/stream.rs` — SSE event parsing (typed events: `message_start`, `content_block_delta`, etc.)
- `codex-rs/anthropic/src/error.rs` — Error types

**New module in core:**

- `codex-rs/core/src/anthropic_bridge.rs` — Anthropic event→`ResponseEvent` mapping, tool schema translation

## Files Modified

**Workspace:**

- `codex-rs/Cargo.toml` — add workspace member + dependency
- `codex-rs/Cargo.lock` — dependency lockfile refresh
- `MODULE.bazel.lock` — Bazel lockfile refresh after dependency changes

**Core (wire dispatch + provider config):**

- `codex-rs/core/src/model_provider_info.rs` — add `AnthropicMessages` to `WireApi`, accept `wire_api = "anthropic_messages"`, add `anthropic` built-in provider with `env_key = "ANTHROPIC_API_KEY"`, and expose a shared provider-header accessor for the Anthropic path
- `codex-rs/core/src/client.rs` — dispatch to Anthropic client when `wire_api == AnthropicMessages`
- `codex-rs/core/src/models_manager/manager.rs` or equivalent session-bootstrap path — fail fast when Anthropic is the selected provider and no explicit Claude model is configured
- `codex-rs/core/src/api_bridge.rs` — no changes (Anthropic path bypasses `CoreAuthProvider`)
- `codex-rs/core/src/anthropic_bridge.rs` — (new) event mapping and tool translation

**NOT modified in stage 3a** (deferred to 3b):

- `codex-rs/core/src/auth.rs` — no new `AuthMode` variants, no `CodexAuth` changes
- `codex-rs/login/` — no new login flows
- `codex-rs/cli/src/login.rs` — no CLI login changes
- `codex-rs/tui/src/onboarding/` — no TUI auth changes
- `codex-rs/tui_app_server/src/onboarding/` — no app-server TUI changes
- `codex-rs/app-server-protocol/` — no `AuthMode`/`LoginAccountParams` changes
- `codex-rs/app-server/` — no login handler changes
- `codex-rs/otel/` — no `TelemetryAuthMode` changes

**Config:**

- `codex-rs/core/models.json` — add valid current-schema Claude model entries with `visibility = "hide"` and no Anthropic-only fields

**Not updated in stage 3a unless scope expands:**

- `codex-rs/core/config.schema.json` — unchanged unless a real typed config shape changes
- `codex-rs/app-server/README.md` — unchanged for the API-key-only explicit-provider slice

---

### Task 1: Create `orbit-code-anthropic` Crate (Wire Protocol)

Implements the Anthropic Messages API (`POST /v1/messages`) with SSE streaming.

- **Step 1: Create Cargo.toml**

```toml
[package]
name = "orbit-code-anthropic"
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

- **Step 2: Define request/response types in `types.rs`**

Match the Anthropic Messages API spec. Key types:

```rust
/// POST /v1/messages request body.
pub struct MessagesRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub system: Option<Vec<SystemBlock>>,
    pub tools: Option<Vec<Tool>>,
    pub tool_choice: Option<ToolChoice>,
    pub thinking: Option<ThinkingConfig>,
    pub max_tokens: u64,
    pub stream: bool,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u64>,
    pub metadata: Option<Metadata>,
}

pub struct Message {
    pub role: Role,                         // "user" or "assistant"
    pub content: Content,                   // String or Vec<ContentBlock>
}

pub enum Content {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

pub enum ContentBlock {
    Text { text: String, cache_control: Option<CacheControl> },
    Image { source: ImageSource },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: String, is_error: Option<bool> },
}
```

- **Step 3: Define SSE event types in `stream.rs`**

Anthropic SSE uses typed event names (unlike Chat Completions which uses generic `data:` lines):

```rust
pub enum AnthropicEvent {
    MessageStart { message_id: String, model: String, usage: Usage },
    ContentBlockStart { index: u32, block_type: ContentBlockType },
    ContentBlockDelta { index: u32, delta: DeltaType },
    ContentBlockStop { index: u32 },
    MessageDelta { stop_reason: Option<String>, usage: Option<Usage> },
    MessageStop,
    Ping,
    Error { error_type: String, message: String },
}

pub enum ContentBlockType {
    Text { text: String },
    ToolUse { id: String, name: String },
    Thinking { thinking: String },
}

pub enum DeltaType {
    TextDelta { text: String },
    InputJsonDelta { partial_json: String },
    ThinkingDelta { thinking: String },
}
```

SSE event name → type mapping:

- `event: message_start` → `AnthropicEvent::MessageStart`
- `event: content_block_start` → `AnthropicEvent::ContentBlockStart`
- `event: content_block_delta` → `AnthropicEvent::ContentBlockDelta`
- `event: content_block_stop` → `AnthropicEvent::ContentBlockStop`
- `event: message_delta` → `AnthropicEvent::MessageDelta`
- `event: message_stop` → `AnthropicEvent::MessageStop`
- `event: ping` → `AnthropicEvent::Ping`
- `event: error` → `AnthropicEvent::Error`
- **Step 4: Implement `AnthropicClient` in `client.rs`**

```rust
pub struct AnthropicClient {
    transport: Arc<dyn HttpTransport>,
    base_url: String,
}

impl AnthropicClient {
    pub async fn stream(
        &self,
        request: MessagesRequest,
        api_key: String,
        extra_headers: HeaderMap,
    ) -> Result<AnthropicStream, AnthropicError>
}
```

If a forward-compatible auth enum is introduced, keep it private to the crate and still expose only API-key behavior in stage 3a.

**Headers to set:**

- Always: `anthropic-version: 2023-06-01`
- Always: `Content-Type: application/json`
- Always: `anthropic-beta: claude-code-20250219,interleaved-thinking-2025-05-14,fine-grained-tool-streaming-2025-05-14`
- For `claude-sonnet-4-6` and `claude-opus-4-6`: append `context-1m-2025-08-07` to `anthropic-beta`
- API key mode only in 3a: `x-api-key: {key}`

**Message sanitization:**

- **Anthropic rejects empty content** — filter out empty string messages and remove empty text/reasoning parts from array content. Messages whose content becomes empty after filtering must be removed entirely.
- **toolCallId sanitization** — Claude requires `toolCallId` values to contain only `[a-zA-Z0-9_-]`. Replace invalid chars: `.replace(/[^a-zA-Z0-9_-]/g, "_")`

**Thinking/reasoning defaults (keep in code, not in `models.json`):**

- Opus 4.6 and Sonnet 4.6 use **adaptive thinking**: `{ type: "adaptive" }` with effort levels `["low", "medium", "high", "max"]`
- Older Claude models use **budgeted thinking**: `{ type: "enabled", budgetTokens: N }` where N is capped at `min(16000, output_limit/2 - 1)` for high, `min(31999, output_limit - 1)` for max

Use a helper like `anthropic_model_defaults(slug: &str) -> AnthropicModelDefaults` so `max_tokens`, thinking config, and 1M-context beta requirements stay in code until the shared model contract grows.

- **Step 5: Define error types in `error.rs`**

```rust
#[derive(Debug, Error)]
pub enum AnthropicError {
    #[error(transparent)]
    Transport(#[from] orbit_code_client::TransportError),
    #[error("Anthropic API error ({status}): [{error_type}] {message}")]
    Api { status: u16, error_type: String, message: String },
    #[error("SSE parse error: {0}")]
    StreamParse(String),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("Anthropic overloaded (529)")]
    Overloaded,
    #[error("Rate limited (429)")]
    RateLimited,
}
```

- **Step 5a: Box the `Api` error variant to stay under 256-byte large-error-threshold**

```rust
#[derive(Debug)]
pub struct AnthropicApiError {
    pub status: u16,
    pub error_type: String,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum AnthropicError {
    // ...
    #[error("Anthropic API error ({status}): [{error_type}] {message}", status = .0.status, error_type = .0.error_type, message = .0.message)]
    Api(Box<AnthropicApiError>),
    // ...
}
```

- **Step 6: Wire up `lib.rs`**
- **Step 7: Add to workspace**

In `codex-rs/Cargo.toml`, add to members and workspace.dependencies:

```toml
# members
"anthropic",
# workspace.dependencies
orbit-code-anthropic = { path = "anthropic" }
```

- **Step 7a: Create BUILD.bazel**

Create `codex-rs/anthropic/BUILD.bazel` with a `rust_library` target, deps matching Cargo.toml, and test targets. Then run:

```bash
just bazel-lock-update
just bazel-lock-check
```

- **Step 8: cargo check + format + commit**

```bash
cd codex-rs && cargo check -p orbit-code-anthropic
just fmt
just fix -p orbit-code-anthropic
git add codex-rs/anthropic/ codex-rs/Cargo.toml codex-rs/Cargo.lock
git commit -m "Add orbit-code-anthropic crate: Anthropic Messages API SSE client"
```

---

### Task 2: Keep OAuth and Auth Storage Deferred

- Do not add Anthropic OAuth code, login CLI changes, TUI onboarding changes, app-server auth changes, or provider-scoped auth storage in stage 3a.
- Keep all OAuth-only headers, tool prefixing behavior, and persisted auth formats out of the 3a implementation.
- Use `docs/migration/03b-anthropic-oauth.md` for the follow-on OAuth and auth-storage work.

---

### Task 3: Wire Claude into Core (Provider Config + Dispatch)

- **Step 1: Extend `WireApi` enum**

In `codex-rs/core/src/model_provider_info.rs`:

```rust
pub enum WireApi {
    #[default]
    Responses,
    /// Anthropic Messages API at `/v1/messages`.
    AnthropicMessages,
}
```

Custom provider configs must use `wire_api = "anthropic_messages"` exactly. Update both `Display` and `Deserialize` so the new variant round-trips cleanly and rejects unknown spellings.

- **Step 2: Add `anthropic` built-in provider**

In the built-in provider definitions (same file or wherever `openai`/`ollama`/`lmstudio` are defined):

```rust
ModelProviderInfo {
    name: "Anthropic".to_string(),
    base_url: Some("https://api.anthropic.com".to_string()),
    env_key: Some("ANTHROPIC_API_KEY".to_string()),
    env_key_instructions: Some("Get your API key at https://console.anthropic.com/settings/keys".to_string()),
    wire_api: WireApi::AnthropicMessages,
    http_headers: Some(HashMap::from([
        ("anthropic-version".to_string(), "2023-06-01".to_string()),
        ("anthropic-beta".to_string(),
         "claude-code-20250219,interleaved-thinking-2025-05-14,fine-grained-tool-streaming-2025-05-14".to_string()),
    ])),
    requires_openai_auth: false,
    supports_websockets: false,
    ..Default::default()
}
```

- Promote `ModelProviderInfo::build_header_map()` to `pub(crate)` or add an equivalent shared accessor so the Anthropic dispatch path can reuse the existing static-plus-env header composition logic.

- **Step 3: Add dispatch in `core::client.rs`**

In `ModelClientSession::stream()`, branch on `wire_api`:

```rust
match wire_api {
    WireApi::Responses => {
        // existing ResponsesClient path (unchanged)
    }
    WireApi::AnthropicMessages => {
        self.stream_anthropic_messages(prompt, model_info, session_telemetry).await
    }
}
```

**Auth in stage 3a is simple:** The Anthropic provider uses `env_key = "ANTHROPIC_API_KEY"`, so `ModelProviderInfo::api_key()` returns the key from the environment. The dispatch bypasses `current_client_setup()` / `CoreAuthProvider` (which only supports `Authorization: Bearer`) and passes the key directly to `AnthropicClient`. No `CodexAuth`, `AuthManager`, or `AuthMode` involvement.

```rust
if provider_is_anthropic && requested_model.is_none() {
    return Err(CodexErr::InvalidRequest {
        message: "Anthropic stage 3a requires an explicit Claude model when `model_provider = \"anthropic\"` is set.".to_string(),
    });
}

async fn stream_anthropic_messages(&self, ...) -> Result<ResponseStream> {
    if is_known_anthropic_model(&model_info.slug) && wire_api != WireApi::AnthropicMessages {
        return Err(CodexErr::InvalidRequest {
            message: "Claude models require `model_provider = \"anthropic\"` in stage 3a.".to_string(),
        });
    }

    // 1. Get API key from env via provider.api_key() — no CodexAuth needed
    let key = self.client.state.provider.api_key()?
        .ok_or_else(|| CodexErr::EnvVar(EnvVarError {
            var: "ANTHROPIC_API_KEY".to_string(),
            instructions: self.client.state.provider.env_key_instructions.clone(),
        }))?;

    // 2. Build AnthropicClient with provider.base_url directly (bypass to_api_provider())
    let base_url = self.client.state.provider.base_url
        .clone()
        .unwrap_or_else(|| "https://api.anthropic.com".to_string());
    let defaults = anthropic_model_defaults(&model_info.slug)?;
    let mut extra_headers = self.client.state.provider.build_header_map()?;
    merge_anthropic_beta_headers(&mut extra_headers, defaults.additional_beta_headers);
    let client = AnthropicClient::new(build_reqwest_client(), base_url);

    // 3. Translate tools to Anthropic format
    let anthropic_tools = tools_to_anthropic_format(&prompt.tools)?;

    // 4. Build MessagesRequest
    let request = MessagesRequest {
        model: model_info.slug.clone(),
        messages: /* translate from prompt.input */,
        tools: Some(anthropic_tools),
        thinking: defaults.thinking,
        max_tokens: defaults.max_tokens,
        stream: true,
        temperature: None,
        // ...
    };

    // 5. Stream and map events
    let anthropic_stream = client.stream(request, key, extra_headers).await?;
    let mapped = map_anthropic_to_response_events(anthropic_stream);
    Ok(mapped)
}
```

- **Step 3a: Create `anthropic_bridge.rs` — Tool schema translation**

OpenAI Responses API tools and Anthropic Messages API tools use different schemas. Create `core/src/anthropic_bridge.rs` with the translation:

```rust
//! Bridge between internal Codex types and the Anthropic Messages API.
//!
/// Convert internal tool definitions to Anthropic tool format.
///
/// OpenAI format: { "type": "function", "name": "...", "description": "...", "parameters": { JSON Schema } }
/// Anthropic format: { "name": "...", "description": "...", "input_schema": { JSON Schema } }
pub fn tools_to_anthropic_format(tools: &[ToolSpec]) -> Result<Vec<anthropic::Tool>> {
    tools
        .iter()
        .map(|tool| match tool {
            ToolSpec::Function(tool) => Ok(anthropic::Tool {
                name: tool.name.clone(),
                description: tool.description.clone(),
                input_schema: tool.parameters.clone(),
            }),
            ToolSpec::ToolSearch { .. }
            | ToolSpec::LocalShell {}
            | ToolSpec::ImageGeneration { .. }
            | ToolSpec::WebSearch { .. }
            | ToolSpec::Freeform(_) => Err(CodexErr::InvalidRequest {
                message: format!(
                    "Anthropic stage 3a supports only function tools; received `{}`.",
                    tool.name()
                ),
            }),
        })
        .collect()
}

/// Filter empty content that Anthropic rejects.
pub fn sanitize_messages(messages: &mut Vec<Message>) {
    messages.retain(|msg| {
        match &msg.content {
            Content::Text(s) => !s.is_empty(),
            Content::Blocks(blocks) => !blocks.is_empty(),
        }
    });
    for msg in messages {
        if let Content::Blocks(blocks) = &mut msg.content {
            blocks.retain(|block| match block {
                ContentBlock::Text { text, .. } => !text.is_empty(),
                _ => true,
            });
        }
    }
}

/// Sanitize toolCallId to only [a-zA-Z0-9_-] per Claude requirements.
pub fn sanitize_tool_call_id(id: &str) -> String {
    id.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' }).collect()
}
```

- **Step 3b: Create `anthropic_bridge.rs` — Event mapping (AnthropicEvent → ResponseEvent)**

This is the core translation layer. Anthropic SSE events have no 1:1 mapping to `ResponseEvent`/`ResponseItem`. The mapping requires stateful accumulation (tool use JSON fragments must be assembled before emitting a `FunctionCall` item).

```rust
/// Maps a stream of `AnthropicEvent` into `ResponseEvent` using stateful accumulation.
///
/// Mapping table:
///   AnthropicEvent::MessageStart       → ResponseEvent::Created, store message_id for later use as response_id
///   AnthropicEvent::ContentBlockStart(Text)    → no event (prepare text accumulator)
///   AnthropicEvent::ContentBlockDelta(TextDelta)  → ResponseEvent::OutputTextDelta { text }
///   AnthropicEvent::ContentBlockStop (text)    → ResponseEvent::OutputItemDone(ResponseItem::Message { text })
///   AnthropicEvent::ContentBlockStart(ToolUse) → no event (prepare tool call accumulator with id, name)
///   AnthropicEvent::ContentBlockDelta(InputJsonDelta) → accumulate partial JSON
///   AnthropicEvent::ContentBlockStop (tool_use) → ResponseEvent::OutputItemDone(ResponseItem::FunctionCall { id, name, arguments })
///   AnthropicEvent::ContentBlockStart(Thinking)  → no event (prepare thinking accumulator)
///   AnthropicEvent::ContentBlockDelta(ThinkingDelta) → ResponseEvent::ReasoningContentDelta { delta, content_index }
///   AnthropicEvent::ContentBlockStop (thinking)  → no separate event (reasoning is streamed via deltas)
///   AnthropicEvent::MessageDelta       → extract stop_reason + output usage
///   AnthropicEvent::MessageStop        → ResponseEvent::Completed { response_id: message_id, token_usage }
///   AnthropicEvent::Ping               → ignored
///   AnthropicEvent::Error              → map to AnthropicError, abort stream
///
/// Usage accounting:
///   - input_tokens from MessageStart.usage
///   - output_tokens from MessageDelta.usage (cumulative)
///   - cached_input_tokens: not available (set to 0)
///   - reasoning_output_tokens: count tokens from Thinking blocks (estimated, or 0)
///
/// max_tokens note:
///   Anthropic API REQUIRES max_tokens on every request (unlike OpenAI where it's optional).
///   The dispatch must pass a model-appropriate default from `anthropic_model_defaults`,
///   not from new `models.json` fields.
///
/// header safety note:
///   If a selected model requires `context-1m-2025-08-07` and the header helper did not add it,
///   fail request construction before sending the HTTP request.
pub fn map_anthropic_to_response_events(
    events: impl Stream<Item = Result<AnthropicEvent, AnthropicError>>,
) -> ResponseStream {
    // Implementation: stateful stream transformer with ContentBlockAccumulator
}
```

- **Step 4: Add Claude model entries**

Add Claude entries to `codex-rs/core/models.json`, but keep them valid against the **current** shared schema and hidden from the picker.

- Start from an existing current-schema entry in `models.json` and preserve all currently required fields such as `priority`, `base_instructions`, `model_messages`, `default_reasoning_summary`, `truncation_policy`, and `experimental_supported_tools`.
- Set `visibility` to `"hide"` for all Claude entries in stage 3a so they do not appear in the picker or become defaults before provider-aware filtering exists.
- Keep `supported_in_api = true` so explicit `--model claude-*` works once `model_provider = "anthropic"` is selected.
- Do **not** add new Anthropic-only fields such as `provider`, `max_output_tokens`, `thinking`, or `supports_1m_context` to `models.json` in stage 3a.
- Add at least these slugs with correct display names and context windows: `claude-sonnet-4-5-20250514` (`200000`), `claude-sonnet-4-6` (`1000000`), `claude-opus-4-6` (`1000000`), and `claude-haiku-4-5-20251001` (`200000`).
- Keep Anthropic-specific request defaults in code via the slug-based helper described above.

- **Deferred note: Anthropic auth modes move to Stage 3b**

> No changes to `AuthMode`, `CodexAuth`, `AuthManager`, `UnauthorizedRecovery`, `TelemetryAuthMode`, or any auth storage in stage 3a.
>
> Anthropic API key auth in stage 3a works entirely through the existing `ModelProviderInfo.env_key` mechanism — the built-in `anthropic` provider declares `env_key = "ANTHROPIC_API_KEY"`, and `ModelProviderInfo::api_key()` reads it from the environment. The dispatch in `client.rs` passes this key directly to `AnthropicClient`. No `CodexAuth` involvement at all.
>
> Stage 3b will add `AuthMode::AnthropicApiKey` / `AnthropicOAuth`, `CodexAuth` variants, persisted auth storage, OAuth refresh, and all the exhaustive match updates across `auth.rs`, `client.rs`, `otel`, `app-server-protocol`, TUI, and CLI.

- **Step 5: cargo check + format + commit**

```bash
cd codex-rs && cargo check
just fmt
just fix -p orbit-code-core
git add codex-rs/core/ codex-rs/Cargo.toml codex-rs/Cargo.lock
git commit -m "Wire Claude provider: WireApi::AnthropicMessages, model entries, env key auth"
```

---

### Task 4: Add Unit Tests

- **Step 1: SSE parsing tests for `orbit-code-anthropic`**

Test the full SSE event sequence:

- `message_start` → `content_block_start(text)` → `content_block_delta(text)` → `content_block_stop` → `message_delta` → `message_stop`
- Tool use: `content_block_start(tool_use)` → `content_block_delta(input_json)` → `content_block_stop`
- Thinking: `content_block_start(thinking)` → `content_block_delta(thinking)` → `content_block_stop`
- Error event parsing
- Usage extraction from `message_start` and `message_delta`
- **Step 2: `WireApi` and picker visibility tests**

Verify:

- `wire_api = "anthropic_messages"` deserializes correctly and round-trips through `Display`
- Claude bundled presets map to `show_in_picker = false`
- Default model selection remains unchanged for non-Anthropic users
- `model_provider = "anthropic"` without explicit `model` fails fast with the documented stage-`3a` error

- **Step 3: Auth header tests**

Verify:

- API key mode sends `x-api-key` and no `Authorization`
- 4.6 / 1M models append `context-1m-2025-08-07`
- `temperature` is omitted from the request payload
- Missing required 1M beta header fails request construction before any HTTP call

- **Step 4: Event mapping tests (in `core`)**

Test the `anthropic_bridge::map_anthropic_to_response_events` mapping:

- Text response: `MessageStart → ContentBlockStart(Text) → ContentBlockDelta(TextDelta) × N → ContentBlockStop → MessageDelta → MessageStop` produces correct `OutputTextDelta` + `OutputItemDone` + `Completed` events
- Tool use: `ContentBlockStart(ToolUse) → ContentBlockDelta(InputJsonDelta) × N → ContentBlockStop` produces `OutputItemDone(FunctionCall)` with assembled JSON arguments
- Thinking: `ContentBlockStart(Thinking) → ContentBlockDelta(ThinkingDelta) × N → ContentBlockStop` produces `ReasoningContentDelta` events
- Mixed: text + tool_use + thinking in same message produces interleaved events in correct order
- Error mid-stream: `Error` event aborts stream with mapped error
- Usage: `message_start.usage.input_tokens` + `message_delta.usage.output_tokens` are correctly extracted
- Unsupported tools: `ToolSpec::ToolSearch`, `LocalShell`, `ImageGeneration`, `WebSearch`, and `Freeform` fail with `Anthropic stage 3a supports only function tools; received \`{tool_name}\`.`
- Provider mismatch: selecting a known Claude model while the provider is not Anthropic fails fast with `Claude models require \`model_provider = "anthropic"\` in stage 3a.`

- **Step 5: Run tests + commit**

```bash
cd codex-rs && cargo test -p orbit-code-anthropic
cd codex-rs && cargo test -p orbit-code-core -- anthropic
just fmt
git add codex-rs/anthropic/ codex-rs/core/
git commit -m "Add unit tests for Anthropic SSE parsing, auth headers, model gating, event mapping"
```

---

### Task 5: Verify and Finalize

- **Step 1: Full workspace check + lint**

```bash
cd codex-rs && cargo check
just fmt
just fix -p orbit-code-anthropic
just fix -p orbit-code-core
```

- **Step 2: Run affected crate tests**

```bash
cd codex-rs && cargo test -p orbit-code-anthropic
cd codex-rs && cargo test -p orbit-code-core
cd codex-rs && cargo test -p orbit-code-cli
```

- **Step 3: Lockfile and Bazel maintenance**

```bash
cd codex-rs && cargo generate-lockfile
just bazel-lock-update
just bazel-lock-check
```

- **Step 4: Final commit**

```bash
git add codex-rs/anthropic/ codex-rs/core/ codex-rs/Cargo.toml codex-rs/Cargo.lock
git commit -m "Stage 3a complete: Claude provider with API key auth"
```

---

## Edge Cases to Handle

These were identified during audit and must be addressed during implementation:

- **Dual credentials:** If both `ANTHROPIC_API_KEY` and `OPENAI_API_KEY` are set, provider routing is determined by `model_provider` in config. Both env vars can coexist safely — the provider's `env_key` determines which one is read.
- **Explicit provider and model required:** In stage 3a, `--model claude-sonnet-4-6` will NOT auto-detect the provider, and `model_provider = "anthropic"` will NOT auto-select a Claude default. User must set `model_provider = "anthropic"` and select an explicit Claude model.
- **Missing explicit model:** If `model_provider = "anthropic"` is set but no explicit Claude model is selected, fail fast with `Anthropic stage 3a requires an explicit Claude model when \`model_provider = "anthropic"\` is set.`
- **Provider/model mismatch:** If the user selects a known Claude model while `model_provider` still resolves to `openai`, fail fast with `Claude models require \`model_provider = "anthropic"\` in stage 3a.` instead of sending the request to the wrong transport.
- **Hidden picker behavior:** Claude bundled presets remain hidden until provider-aware filtering exists. Manual model entry is the only supported selection path in 3a.
- **Anthropic 529 (overloaded):** Must trigger exponential backoff with ±10% jitter per convention 17. Wire `AnthropicError::Overloaded` into the core retry loop alongside the existing 429/5xx handling.
- **`max_tokens` is required:** Anthropic API rejects requests without `max_tokens`. The dispatch must always set it. Hardcode per-model defaults until the shared model metadata contract grows.
- **Empty content blocks:** Anthropic rejects messages with empty `content` arrays. The message translation layer must filter these out. Also filter empty text and reasoning parts within array content.
- **toolCallId sanitization:** Claude requires tool call IDs to match `[a-zA-Z0-9_-]` only. Invalid characters must be replaced with `_` before sending.
- **Anthropic SSE `error` event mid-stream:** An `error` event should abort the stream and map to `AnthropicError::Api`. It is not recoverable within the same stream.
- **Temperature:** Claude models should use `None` for temperature, letting Anthropic use its default. Do not send a temperature value.
- **Tool subset for stage 3a:** Support only `ToolSpec::Function`. Reject `ToolSearch`, `LocalShell`, `ImageGeneration`, `WebSearch`, and freeform tools such as `apply_patch` with `Anthropic stage 3a supports only function tools; received \`{tool_name}\`.`
- **1M context beta header:** If a model needs 1M context, the client MUST send `anthropic-beta: context-1m-2025-08-07`. Failing to do so would silently fall back to 200K, so treat it as an implementation bug and fail request construction before any HTTP call.
- **Accepted `wire_api` spelling:** Custom provider config must use `wire_api = "anthropic_messages"`.

## Implementation Notes

These are not stage `3a` scope changes, but the implementation should keep them consistent:

- **Centralize Anthropic error strings:** Define shared constants or helpers for the stage `3a` Anthropic configuration/tooling errors so CLI, TUI, app startup, and any other startup surfaces do not drift in wording.
- **Unsupported-tool error text:** Keep the current stage `3a` message stable unless implementation constraints require a small wording change: `Anthropic stage 3a supports only function tools; received \`{tool_name}\`.`
- **Wrong-provider error text:** Keep the current stage `3a` mismatch message stable across all startup surfaces unless implementation constraints require a small wording change: `Claude models require \`model_provider = "anthropic"\` in stage 3a.`
- **Missing-model error text:** Keep the current stage `3a` missing-model message stable across all startup surfaces unless implementation constraints require a small wording change: `Anthropic stage 3a requires an explicit Claude model when \`model_provider = "anthropic"\` is set.`
- **Slug-helper maintenance:** Any new Claude slug added during or after stage `3a` must update the slug-based `anthropic_model_defaults` helper, the 1M-context header logic, and the targeted tests that cover Anthropic defaults. Do not add new Claude entries to `models.json` without updating that helper in the same change.

---

## Expected Outcomes

After Stage 3a:

- Users can set `model_provider = "anthropic"`, `ANTHROPIC_API_KEY`, and an explicit Claude model
- `orbit-code --model claude-sonnet-4-6` works with explicit provider config
- `model_provider = "anthropic"` without an explicit Claude model fails fast instead of falling through to a non-Claude default
- Claude model presets are bundled but hidden from the picker in stage 3a
- SSE streaming works with Anthropic's typed event protocol
- Function tool calling works for the supported `ToolSpec::Function` subset
- Thinking/reasoning blocks are parsed and surfaced via `ReasoningContentDelta`
- 1M context enabled for Sonnet 4.6 and Opus 4.6 via beta header
- No changes to login, auth storage, TUI, app-server protocol, or shared `AuthMode`

## What's Next

- **Stage 3b:** Anthropic OAuth, provider-scoped auth storage, login/TUI/app-server redesign
- **Stage 3c:** OpenRouter provider (Chat Completions API)
