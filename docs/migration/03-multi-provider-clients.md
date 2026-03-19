# Stage 3: Multi-Provider Client Layer

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build three new Rust HTTP client crates -- `orbit-chat-completions`, `orbit-anthropic`, and `orbit-gemini` -- that implement SSE streaming for the OpenAI Chat Completions, Anthropic Messages, and Google GenerateContent wire protocols respectively, reusing the existing `orbit-code-client` HTTP transport layer.

**Architecture:** Each crate follows the same pattern as `codex-api` (now `orbit-code-api`): it defines request/response types, constructs HTTP requests, and processes SSE streams into a channel of typed events. All three crates depend on `orbit-code-client` for `HttpTransport`, `ReqwestTransport`, retry, SSE parsing, and custom CA support. They output events in a common `ResponseEvent`-compatible format so `core` can consume them uniformly.

**Tech Stack:** Rust, `reqwest`, `eventsource-stream`, `serde`/`serde_json`, `tokio`, `futures`.

**Depends on:** Stage 2 (dead crates removed, clean workspace).

---

## Reference: TypeScript API Shapes

The TypeScript Agent-backend at `Snowflake-v0/Agent-backend/packages/opencode/src/provider/provider.ts` uses the Vercel AI SDK which abstracts wire protocols behind `@ai-sdk/openai`, `@ai-sdk/anthropic`, `@ai-sdk/google`, etc. Our Rust crates implement the raw HTTP wire protocols directly. The three wire protocols are:

1. **Chat Completions** (`/v1/chat/completions`) -- Used by OpenRouter, Groq, Mistral, DeepSeek, xAI, Together, Perplexity, Cerebras, DeepInfra, Azure, Qwen, GLM, Ollama, LM Studio, GitHub Copilot, GitLab.
2. **Anthropic Messages** (`/v1/messages`) -- Used by Anthropic (Claude models).
3. **Google GenerateContent** (`/v1beta/models/{model}:streamGenerateContent`) -- Used by Gemini and Vertex AI.

---

## Files Created

**New crate: `orbit-chat-completions`**
- `codex-rs/orbit-chat-completions/Cargo.toml`
- `codex-rs/orbit-chat-completions/src/lib.rs`
- `codex-rs/orbit-chat-completions/src/types.rs` -- Request/response serde types
- `codex-rs/orbit-chat-completions/src/client.rs` -- `ChatCompletionsClient` with SSE streaming
- `codex-rs/orbit-chat-completions/src/stream.rs` -- SSE event parsing into typed events
- `codex-rs/orbit-chat-completions/src/error.rs` -- Error types

**New crate: `orbit-anthropic`**
- `codex-rs/orbit-anthropic/Cargo.toml`
- `codex-rs/orbit-anthropic/src/lib.rs`
- `codex-rs/orbit-anthropic/src/types.rs` -- Request/response serde types
- `codex-rs/orbit-anthropic/src/client.rs` -- `AnthropicClient` with SSE streaming
- `codex-rs/orbit-anthropic/src/stream.rs` -- SSE event parsing into typed events
- `codex-rs/orbit-anthropic/src/error.rs` -- Error types

**New crate: `orbit-gemini`**
- `codex-rs/orbit-gemini/Cargo.toml`
- `codex-rs/orbit-gemini/src/lib.rs`
- `codex-rs/orbit-gemini/src/types.rs` -- Request/response serde types
- `codex-rs/orbit-gemini/src/client.rs` -- `GeminiClient` with SSE streaming
- `codex-rs/orbit-gemini/src/stream.rs` -- SSE event parsing into typed events
- `codex-rs/orbit-gemini/src/error.rs` -- Error types

**Modified:**
- `codex-rs/Cargo.toml` -- Add 3 new workspace members and dependencies

---

### Task 1: Create `orbit-chat-completions` Crate

This crate implements the OpenAI Chat Completions API (`/v1/chat/completions`) with streaming (SSE). This is the most widely used wire protocol -- OpenRouter, Groq, Mistral, DeepSeek, xAI, Together, Perplexity, Cerebras, DeepInfra, Azure, Ollama, LM Studio, and any OpenAI-compatible endpoint all speak this protocol.

**Files:**
- Create: `codex-rs/orbit-chat-completions/Cargo.toml`
- Create: `codex-rs/orbit-chat-completions/src/lib.rs`
- Create: `codex-rs/orbit-chat-completions/src/types.rs`
- Create: `codex-rs/orbit-chat-completions/src/client.rs`
- Create: `codex-rs/orbit-chat-completions/src/stream.rs`
- Create: `codex-rs/orbit-chat-completions/src/error.rs`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "orbit-chat-completions"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
orbit-code-client = { workspace = true }
eventsource-stream = { workspace = true }
futures = { workspace = true }
http = { workspace = true }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
thiserror = { workspace = true }
tokio = { workspace = true, features = ["sync"] }
tracing = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }
```

- [ ] **Step 2: Define request/response types in `types.rs`**

Types to define (matching the OpenAI Chat Completions API spec):

```rust
// Request types
pub struct ChatCompletionsRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub tool_choice: Option<ToolChoice>,
    pub stream: bool,
    pub stream_options: Option<StreamOptions>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<u64>,
    pub max_completion_tokens: Option<u64>,
    pub reasoning_effort: Option<String>,
    pub parallel_tool_calls: Option<bool>,
}

pub struct ChatMessage {
    pub role: MessageRole,          // system, user, assistant, tool
    pub content: ChatContent,       // String or Vec<ContentPart>
    pub name: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

pub enum ChatContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

pub enum ContentPart {
    Text { text: String },
    ImageUrl { image_url: ImageUrl },
}

pub struct ToolCall {
    pub id: String,
    pub r#type: String,             // "function"
    pub function: FunctionCall,
}

pub struct FunctionCall {
    pub name: String,
    pub arguments: String,          // JSON string
}

pub struct StreamOptions {
    pub include_usage: bool,
}

// Response/stream types
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChunkChoice>,
    pub usage: Option<Usage>,
}

pub struct ChunkChoice {
    pub index: u32,
    pub delta: ChunkDelta,
    pub finish_reason: Option<String>,
}

pub struct ChunkDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ChunkToolCall>>,
    pub reasoning_content: Option<String>,  // DeepSeek, QwQ
}

pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}
```

- [ ] **Step 3: Implement `ChatCompletionsClient` in `client.rs`**

The client takes `HttpTransport` + `AuthProvider`-style auth (bearer token string) and builds HTTP requests:

- `POST {base_url}/chat/completions`
- Headers: `Authorization: Bearer {key}`, `Content-Type: application/json`, `Accept: text/event-stream`
- Body: serialized `ChatCompletionsRequest` with `stream: true`
- Returns a channel of `ChatCompletionEvent` values

Key method signature:
```rust
pub async fn stream(
    &self,
    request: ChatCompletionsRequest,
    extra_headers: HeaderMap,
) -> Result<ChatCompletionStream, ChatCompletionsError>
```

Uses `orbit_code_client::sse_stream` to convert the HTTP SSE response into parsed events.

- [ ] **Step 4: Implement SSE parsing in `stream.rs`**

Parse SSE `data:` lines. Each line contains a JSON `ChatCompletionChunk`. The stream ends when a `data: [DONE]` line is received.

Map each chunk into a `ChatCompletionEvent`:
```rust
pub enum ChatCompletionEvent {
    Delta { content: Option<String>, tool_calls: Option<Vec<ChunkToolCall>>, reasoning: Option<String> },
    Usage(Usage),
    Done,
    Error(String),
}
```

- [ ] **Step 5: Define error types in `error.rs`**

```rust
pub enum ChatCompletionsError {
    Transport(orbit_code_client::TransportError),
    Api { status: u16, message: String, code: Option<String> },
    StreamParse(String),
    Json(serde_json::Error),
}
```

- [ ] **Step 6: Wire up `lib.rs` with public exports**

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

- [ ] **Step 7: Commit**

```bash
git add codex-rs/orbit-chat-completions/
git commit -m "Add orbit-chat-completions crate: Chat Completions API SSE client"
```

---

### Task 2: Create `orbit-anthropic` Crate

This crate implements the Anthropic Messages API (`/v1/messages`) with streaming (SSE). The Anthropic SSE protocol differs from Chat Completions -- it uses typed event names (`message_start`, `content_block_start`, `content_block_delta`, `content_block_stop`, `message_delta`, `message_stop`) rather than a single `data:` JSON object per chunk.

**Files:**
- Create: `codex-rs/orbit-anthropic/Cargo.toml`
- Create: `codex-rs/orbit-anthropic/src/lib.rs`
- Create: `codex-rs/orbit-anthropic/src/types.rs`
- Create: `codex-rs/orbit-anthropic/src/client.rs`
- Create: `codex-rs/orbit-anthropic/src/stream.rs`
- Create: `codex-rs/orbit-anthropic/src/error.rs`

- [ ] **Step 1: Create Cargo.toml**

Same dependency structure as `orbit-chat-completions`.

- [ ] **Step 2: Define request/response types in `types.rs`**

Types to define (matching Anthropic Messages API spec):

```rust
// Request
pub struct MessagesRequest {
    pub model: String,
    pub messages: Vec<AnthropicMessage>,
    pub system: Option<Vec<SystemBlock>>,
    pub tools: Option<Vec<AnthropicTool>>,
    pub tool_choice: Option<AnthropicToolChoice>,
    pub max_tokens: u64,
    pub stream: bool,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u64>,
    pub metadata: Option<AnthropicMetadata>,
}

pub struct AnthropicMessage {
    pub role: String,                       // "user" or "assistant"
    pub content: AnthropicContent,          // String or Vec<ContentBlock>
}

pub enum AnthropicContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

pub enum ContentBlock {
    Text { text: String, cache_control: Option<CacheControl> },
    Image { source: ImageSource },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: String, is_error: Option<bool> },
}

pub struct CacheControl {
    pub r#type: String,                     // "ephemeral"
}

pub struct SystemBlock {
    pub r#type: String,                     // "text"
    pub text: String,
    pub cache_control: Option<CacheControl>,
}

// SSE event types
pub struct MessageStart {
    pub r#type: String,
    pub message: MessageStartBody,
}

pub struct ContentBlockStart {
    pub index: u32,
    pub content_block: ContentBlockType,
}

pub enum ContentBlockType {
    Text { text: String },
    ToolUse { id: String, name: String },
    Thinking { thinking: String },
}

pub struct ContentBlockDelta {
    pub index: u32,
    pub delta: DeltaType,
}

pub enum DeltaType {
    TextDelta { text: String },
    InputJsonDelta { partial_json: String },
    ThinkingDelta { thinking: String },
}

pub struct MessageDelta {
    pub delta: MessageDeltaBody,
    pub usage: Option<AnthropicUsage>,
}

pub struct AnthropicUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
}
```

- [ ] **Step 3: Implement `AnthropicClient` in `client.rs`**

- `POST {base_url}/v1/messages`
- Headers: `x-api-key: {key}`, `anthropic-version: 2023-06-01`, `anthropic-beta: claude-code-20250219,interleaved-thinking-2025-05-14,fine-grained-tool-streaming-2025-05-14`, `Content-Type: application/json`
- Body: serialized `MessagesRequest` with `stream: true`
- Returns a channel of `AnthropicEvent` values

Note: Anthropic uses `x-api-key` header instead of `Authorization: Bearer`. The client must handle this difference.

- [ ] **Step 4: Implement SSE parsing in `stream.rs`**

Anthropic SSE uses named events. Parse the `event:` field to determine the event type, then parse the `data:` field accordingly:

```rust
pub enum AnthropicEvent {
    MessageStart(MessageStart),
    ContentBlockStart(ContentBlockStart),
    ContentBlockDelta(ContentBlockDelta),
    ContentBlockStop { index: u32 },
    MessageDelta(MessageDelta),
    MessageStop,
    Ping,
    Error { error_type: String, message: String },
}
```

The SSE event names to handle:
- `message_start` -- Contains model, id, usage
- `content_block_start` -- Start of text/tool_use/thinking block
- `content_block_delta` -- Incremental text/JSON/thinking delta
- `content_block_stop` -- End of a content block
- `message_delta` -- Final stop_reason and usage
- `message_stop` -- Stream complete
- `ping` -- Keep-alive
- `error` -- API error

- [ ] **Step 5: Define error types in `error.rs`**

```rust
pub enum AnthropicError {
    Transport(orbit_code_client::TransportError),
    Api { status: u16, error_type: String, message: String },
    StreamParse(String),
    Json(serde_json::Error),
    Overloaded,     // 529 status
    RateLimited,    // 429 status
}
```

- [ ] **Step 6: Wire up `lib.rs` with public exports**

- [ ] **Step 7: Commit**

```bash
git add codex-rs/orbit-anthropic/
git commit -m "Add orbit-anthropic crate: Anthropic Messages API SSE client"
```

---

### Task 3: Create `orbit-gemini` Crate

This crate implements the Google Gemini GenerateContent API with streaming. Gemini uses a different streaming format -- JSON chunks separated by newlines (not SSE), wrapped in an array structure.

**Files:**
- Create: `codex-rs/orbit-gemini/Cargo.toml`
- Create: `codex-rs/orbit-gemini/src/lib.rs`
- Create: `codex-rs/orbit-gemini/src/types.rs`
- Create: `codex-rs/orbit-gemini/src/client.rs`
- Create: `codex-rs/orbit-gemini/src/stream.rs`
- Create: `codex-rs/orbit-gemini/src/error.rs`

- [ ] **Step 1: Create Cargo.toml**

Same dependency structure as the other two crates.

- [ ] **Step 2: Define request/response types in `types.rs`**

Types to define (matching Google GenerateContent API spec):

```rust
// Request
pub struct GenerateContentRequest {
    pub contents: Vec<GeminiContent>,
    pub tools: Option<Vec<GeminiTool>>,
    pub tool_config: Option<ToolConfig>,
    pub system_instruction: Option<GeminiContent>,
    pub generation_config: Option<GenerationConfig>,
    pub safety_settings: Option<Vec<SafetySetting>>,
}

pub struct GeminiContent {
    pub role: Option<String>,               // "user" or "model"
    pub parts: Vec<GeminiPart>,
}

pub enum GeminiPart {
    Text { text: String },
    InlineData { inline_data: InlineData },
    FunctionCall { function_call: GeminiFunctionCall },
    FunctionResponse { function_response: GeminiFunctionResponse },
}

pub struct GenerationConfig {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u64>,
    pub max_output_tokens: Option<u64>,
    pub candidate_count: Option<u32>,
    pub response_mime_type: Option<String>,
}

pub struct GeminiTool {
    pub function_declarations: Vec<FunctionDeclaration>,
}

pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,      // JSON Schema
}

// Response (streaming chunk)
pub struct GenerateContentResponse {
    pub candidates: Option<Vec<Candidate>>,
    pub usage_metadata: Option<UsageMetadata>,
    pub model_version: Option<String>,
}

pub struct Candidate {
    pub content: Option<GeminiContent>,
    pub finish_reason: Option<String>,
    pub safety_ratings: Option<Vec<SafetyRating>>,
}

pub struct UsageMetadata {
    pub prompt_token_count: Option<u64>,
    pub candidates_token_count: Option<u64>,
    pub total_token_count: Option<u64>,
}
```

- [ ] **Step 3: Implement `GeminiClient` in `client.rs`**

- `POST {base_url}/v1beta/models/{model}:streamGenerateContent?alt=sse&key={api_key}`
- Headers: `Content-Type: application/json`
- Body: serialized `GenerateContentRequest`
- For Vertex AI: `POST https://{location}-aiplatform.googleapis.com/v1/projects/{project}/locations/{location}/publishers/google/models/{model}:streamGenerateContent`
  - Uses OAuth bearer token instead of API key query param

Note: Gemini appends the API key as a query parameter (`key=`) rather than using a header. The client must support both API key (for Gemini) and OAuth bearer (for Vertex AI).

- [ ] **Step 4: Implement stream parsing in `stream.rs`**

Gemini streaming with `alt=sse` returns SSE events where each `data:` line contains a full `GenerateContentResponse` JSON object.

Map each response into a `GeminiEvent`:
```rust
pub enum GeminiEvent {
    ContentDelta { text: Option<String>, function_calls: Option<Vec<GeminiFunctionCall>> },
    Usage(UsageMetadata),
    Done { finish_reason: String },
    Error { code: u32, message: String, status: String },
}
```

- [ ] **Step 5: Define error types in `error.rs`**

```rust
pub enum GeminiError {
    Transport(orbit_code_client::TransportError),
    Api { status: u16, code: u32, message: String },
    StreamParse(String),
    Json(serde_json::Error),
    SafetyBlocked { reason: String },
}
```

- [ ] **Step 6: Wire up `lib.rs` with public exports**

- [ ] **Step 7: Commit**

```bash
git add codex-rs/orbit-gemini/
git commit -m "Add orbit-gemini crate: Gemini GenerateContent API streaming client"
```

---

### Task 4: Register All Three Crates in Workspace

**Files:**
- Modify: `codex-rs/Cargo.toml`

- [ ] **Step 1: Add to workspace members**

Add to the `[workspace] members` array in `codex-rs/Cargo.toml`:
```toml
"orbit-chat-completions",
"orbit-anthropic",
"orbit-gemini",
```

- [ ] **Step 2: Add to workspace dependencies**

Add to `[workspace.dependencies]` in `codex-rs/Cargo.toml`:
```toml
orbit-chat-completions = { path = "orbit-chat-completions" }
orbit-anthropic = { path = "orbit-anthropic" }
orbit-gemini = { path = "orbit-gemini" }
```

- [ ] **Step 3: cargo check**

```bash
cd codex-rs && cargo check -p orbit-chat-completions -p orbit-anthropic -p orbit-gemini
```

Expected: Clean compilation for all three crates.

- [ ] **Step 4: Commit**

```bash
git add codex-rs/Cargo.toml codex-rs/Cargo.lock
git commit -m "Register orbit-chat-completions, orbit-anthropic, orbit-gemini in workspace"
```

---

### Task 5: Add Unit Tests for Each Client

- [ ] **Step 1: Add tests for Chat Completions SSE parsing**

Create `codex-rs/orbit-chat-completions/src/stream_tests.rs` with fixture-based tests:
- Parse a recorded SSE stream from a real Chat Completions response
- Verify text deltas are extracted correctly
- Verify tool call deltas with function arguments are assembled
- Verify `[DONE]` sentinel terminates the stream
- Verify usage is extracted from the final chunk

- [ ] **Step 2: Add tests for Anthropic SSE parsing**

Create `codex-rs/orbit-anthropic/src/stream_tests.rs` with fixture-based tests:
- Parse `message_start` -> `content_block_start` -> `content_block_delta` -> `content_block_stop` -> `message_delta` -> `message_stop`
- Verify tool_use blocks with JSON input deltas are assembled
- Verify thinking blocks are parsed
- Verify usage from `message_delta` is extracted

- [ ] **Step 3: Add tests for Gemini stream parsing**

Create `codex-rs/orbit-gemini/src/stream_tests.rs` with fixture-based tests:
- Parse streaming JSON array chunks
- Verify text candidates are extracted
- Verify function call responses are parsed
- Verify safety ratings and finish reasons are handled

- [ ] **Step 4: Run all tests**

```bash
cd codex-rs && cargo test -p orbit-chat-completions -p orbit-anthropic -p orbit-gemini
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "Add unit tests for multi-provider SSE clients"
```

---

### Task 6: Final Verification

- [ ] **Step 1: Full workspace check**

```bash
cd codex-rs && cargo check
```

Expected: Clean compilation for the entire workspace including the 3 new crates.

- [ ] **Step 2: Verify crate structure**

```bash
for crate in orbit-chat-completions orbit-anthropic orbit-gemini; do
  echo "=== $crate ==="
  ls codex-rs/$crate/src/
done
```

Expected: Each has `lib.rs`, `types.rs`, `client.rs`, `stream.rs`, `error.rs`.

- [ ] **Step 3: Commit and push**

```bash
git add -A
git commit -m "Stage 3 complete: multi-provider client layer (Chat Completions, Anthropic, Gemini)"
git push origin main
```

---

## Expected Outcomes

After Stage 3:
- Three new crates exist in the workspace, each implementing a distinct wire protocol
- `orbit-chat-completions` handles 15+ providers that speak the Chat Completions API
- `orbit-anthropic` handles Claude models with their unique SSE event format
- `orbit-gemini` handles Gemini and Vertex AI with their streaming format
- All three reuse `orbit-code-client` for HTTP transport, retry, custom CA, and SSE parsing
- All three have unit tests with fixture-based SSE stream verification
- The existing `orbit-code-api` (Responses API) remains unchanged and continues working for OpenAI
- The workspace compiles cleanly with ~1,600 lines of new code across the three crates
