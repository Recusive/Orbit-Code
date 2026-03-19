# Stage 7: Core Integration

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Wire the multi-provider client crates (Stage 3), provider registry (Stage 4), auth system (Stage 5), and message normalization (Stage 6) into the `orbit-code-core` engine so the agent loop can dispatch requests to any provider based on configuration.

**Architecture:** The `ModelClient` in `core/src/client.rs` currently talks exclusively to the Responses API via `orbit-code-api`. We add a `ProviderRouter` that sits between the core agent loop and the protocol-specific clients. The router reads the active `WireProtocol` from the provider config, normalizes messages via the `MessageNormalizer`, dispatches to the appropriate client crate, and maps the provider-specific response events back into the common `ResponseEvent` stream that the rest of core already consumes. The existing Responses API path remains the default and is completely unchanged.

**Tech Stack:** Rust, `tokio`, `futures`, `serde_json`.

**Depends on:** Stages 3, 4, 5, 6 (all must be complete).

---

## Files Modified

**Core crate:**
- `codex-rs/core/Cargo.toml` -- Add dependencies on 4 new crates
- `codex-rs/core/src/client.rs` -- Add `WireProtocol` routing branch in `stream_responses`
- `codex-rs/core/src/client_common.rs` -- Extend `ResponseStream` for multi-protocol events
- `codex-rs/core/src/model_provider_info.rs` -- Replace `WireApi` with `WireProtocol` from `orbit-providers`
- `codex-rs/core/src/config/mod.rs` -- Wire provider registry into config loading
- `codex-rs/core/src/default_client.rs` -- Add provider-specific client construction

**New files in core:**
- `codex-rs/core/src/provider_router.rs` -- Central dispatch from core to protocol-specific clients
- `codex-rs/core/src/provider_event_mapper.rs` -- Map provider-specific events to common `ResponseEvent`

**Context manager:**
- `codex-rs/core/src/context_manager/history.rs` -- Update token counting for non-OpenAI models

---

### Task 1: Add New Crate Dependencies to Core

**Files:**
- Modify: `codex-rs/core/Cargo.toml`

- [ ] **Step 1: Add workspace dependencies**

Add to `[dependencies]` in `codex-rs/core/Cargo.toml`:
```toml
orbit-chat-completions = { workspace = true }
orbit-anthropic = { workspace = true }
orbit-gemini = { workspace = true }
orbit-providers = { workspace = true }
```

- [ ] **Step 2: Verify compilation**

```bash
cd codex-rs && cargo check -p orbit-code-core
```

- [ ] **Step 3: Commit**

```bash
git add codex-rs/core/Cargo.toml
git commit -m "Add multi-provider crate dependencies to orbit-code-core"
```

---

### Task 2: Replace `WireApi` with `WireProtocol`

**Files:**
- Modify: `codex-rs/core/src/model_provider_info.rs`

- [ ] **Step 1: Import WireProtocol from orbit-providers**

Replace the existing single-variant `WireApi` enum (currently only has `Responses`) with a re-export of the `WireProtocol` enum from `orbit-providers`:

```rust
// Remove the old WireApi enum definition (lines 38-67 in model_provider_info.rs)
// Replace with:
pub use orbit_providers::wire_protocol::WireProtocol;

// Keep WireApi as a type alias for backward compatibility during transition
pub type WireApi = WireProtocol;
```

- [ ] **Step 2: Update `ModelProviderInfo.wire_api` field**

Change the field type from `WireApi` to `WireProtocol` in the `ModelProviderInfo` struct:
```rust
pub struct ModelProviderInfo {
    // ... existing fields ...
    #[serde(default)]
    pub wire_api: WireProtocol,
    // ... existing fields ...
}
```

- [ ] **Step 3: Update `built_in_model_providers`**

Update the built-in provider definitions to use the new enum variants:
- OpenAI: `WireProtocol::Responses` (unchanged)
- Ollama: `WireProtocol::ChatCompletions` (changed from `Responses` -- now routes through Chat Completions client)
- LM Studio: `WireProtocol::ChatCompletions` (same change)

- [ ] **Step 4: Update deserialization**

Ensure TOML config values `"responses"`, `"chat_completions"`, `"anthropic"`, `"gemini"` are all accepted. The old `"chat"` value should still produce a helpful error.

- [ ] **Step 5: Fix all compilation errors**

Run `cargo check -p orbit-code-core` and fix any code that references the old `WireApi` enum:
- `codex-rs/core/src/client.rs` -- Update match arms
- Any tests referencing `WireApi::Responses`

- [ ] **Step 6: Commit**

```bash
git add codex-rs/core/src/model_provider_info.rs
git commit -m "Replace WireApi with WireProtocol enum supporting 4 wire protocols"
```

---

### Task 3: Create the Provider Router

**Files:**
- Create: `codex-rs/core/src/provider_router.rs`

- [ ] **Step 1: Define the ProviderRouter struct**

```rust
use crate::auth::CodexAuth;
use crate::client_common::ResponseEvent;
use crate::client_common::ResponseStream;
use crate::error::Result;
use crate::model_provider_info::ModelProviderInfo;
use crate::model_provider_info::WireProtocol;
use orbit_code_client::ReqwestTransport;
use orbit_providers::normalize::MessageNormalizer;
use orbit_providers::normalize::NormalizedMessages;
use orbit_providers::normalize::normalizer_for;
use tokio::sync::mpsc;

/// Routes requests to the appropriate wire-protocol client based on
/// the provider's configured WireProtocol.
pub(crate) struct ProviderRouter {
    transport: ReqwestTransport,
}

impl ProviderRouter {
    pub fn new(transport: ReqwestTransport) -> Self {
        Self { transport }
    }

    /// Stream a request through the appropriate provider client.
    ///
    /// This is the main dispatch point. It:
    /// 1. Gets the normalizer for the wire protocol
    /// 2. Converts ResponseItems into wire-specific messages
    /// 3. Sends the request to the appropriate client
    /// 4. Maps response events back to common ResponseEvent
    pub async fn stream(
        &self,
        provider: &ModelProviderInfo,
        auth: &CodexAuth,
        model: &str,
        items: &[ResponseItem],
        tools: &[ToolSpec],
        instructions: &str,
        extra_headers: HeaderMap,
    ) -> Result<ResponseStream> {
        match provider.wire_api {
            WireProtocol::Responses => {
                // Delegate to existing orbit-code-api ResponsesClient
                // This path is UNCHANGED from the current implementation
                self.stream_responses(provider, auth, model, items, tools, instructions, extra_headers).await
            }
            WireProtocol::ChatCompletions => {
                self.stream_chat_completions(provider, auth, model, items, tools, instructions, extra_headers).await
            }
            WireProtocol::Anthropic => {
                self.stream_anthropic(provider, auth, model, items, tools, instructions, extra_headers).await
            }
            WireProtocol::Gemini => {
                self.stream_gemini(provider, auth, model, items, tools, instructions, extra_headers).await
            }
        }
    }
}
```

- [ ] **Step 2: Implement Chat Completions dispatch**

```rust
impl ProviderRouter {
    async fn stream_chat_completions(
        &self,
        provider: &ModelProviderInfo,
        auth: &CodexAuth,
        model: &str,
        items: &[ResponseItem],
        tools: &[ToolSpec],
        instructions: &str,
        extra_headers: HeaderMap,
    ) -> Result<ResponseStream> {
        let normalizer = normalizer_for(WireProtocol::ChatCompletions, /* provider_config */);

        // 1. Normalize messages
        let normalized = normalizer.normalize_messages(items, /* model_def */, /* provider_config */);
        let messages = match normalized {
            NormalizedMessages::ChatCompletions(msgs) => msgs,
            _ => unreachable!("normalizer_for(ChatCompletions) always returns ChatCompletions"),
        };

        // 2. Build request
        let params = normalizer.generation_params(/* model_def */);
        let request = orbit_chat_completions::ChatCompletionsRequest {
            model: model.to_string(),
            messages,
            tools: Some(convert_tools_to_chat_format(tools)),
            stream: true,
            stream_options: Some(StreamOptions { include_usage: true }),
            temperature: params.temperature,
            top_p: params.top_p,
            max_completion_tokens: params.max_output_tokens,
            ..Default::default()
        };

        // 3. Create client and stream
        let api_provider = provider.to_api_provider(None)?;
        let bearer = auth_to_bearer_token(auth);
        let client = orbit_chat_completions::ChatCompletionsClient::new(
            self.transport.clone(),
            &api_provider.base_url,
            bearer.as_deref(),
        );
        let raw_stream = client.stream(request, extra_headers).await?;

        // 4. Map events to common ResponseEvent
        let (tx, rx) = mpsc::channel(32);
        tokio::spawn(async move {
            map_chat_completion_events(raw_stream, tx).await;
        });

        Ok(ResponseStream::from_receiver(rx))
    }
}
```

- [ ] **Step 3: Implement Anthropic dispatch**

Similar structure to Chat Completions but with Anthropic-specific request construction:
- Use `x-api-key` header instead of `Authorization: Bearer`
- Include `anthropic-version` and `anthropic-beta` headers
- Map `AnthropicEvent` variants to `ResponseEvent`

- [ ] **Step 4: Implement Gemini dispatch**

- API key as query parameter (or bearer token for Vertex AI)
- Different URL structure: `/v1beta/models/{model}:streamGenerateContent`
- Map `GeminiEvent` variants to `ResponseEvent`

- [ ] **Step 5: Commit**

```bash
git add codex-rs/core/src/provider_router.rs
git commit -m "Create ProviderRouter with dispatch for all 4 wire protocols"
```

---

### Task 4: Create Provider Event Mapper

**Files:**
- Create: `codex-rs/core/src/provider_event_mapper.rs`

- [ ] **Step 1: Map Chat Completion events to ResponseEvent**

The core engine expects `ResponseEvent` values (defined in `codex-rs/codex-api/src/common.rs`). Map each provider's events:

```rust
/// Map Chat Completions SSE events to the common ResponseEvent format.
pub(crate) async fn map_chat_completion_events(
    mut stream: ChatCompletionStream,
    tx: mpsc::Sender<Result<ResponseEvent, ApiError>>,
) {
    // Accumulate tool calls across deltas (they arrive incrementally)
    let mut tool_call_accumulators: HashMap<u32, ToolCallAccumulator> = HashMap::new();
    let mut response_id = String::new();

    while let Some(event) = stream.next().await {
        match event {
            ChatCompletionEvent::Delta { content, tool_calls, reasoning } => {
                if let Some(text) = content {
                    // Emit as OutputItemDone with a Message ResponseItem
                    let item = ResponseItem::Message {
                        role: "assistant".to_string(),
                        content: vec![ContentItem::OutputText { text }],
                        ..Default::default()
                    };
                    let _ = tx.send(Ok(ResponseEvent::OutputItemAdded(item))).await;
                }
                if let Some(calls) = tool_calls {
                    // Accumulate function call deltas until finish_reason="tool_calls"
                    for call in calls {
                        tool_call_accumulators
                            .entry(call.index)
                            .or_default()
                            .append(&call);
                    }
                }
            }
            ChatCompletionEvent::Usage(usage) => {
                // Will be emitted with Completed event
            }
            ChatCompletionEvent::Done => {
                // Emit accumulated tool calls as FunctionCall items
                for (_, acc) in tool_call_accumulators.drain() {
                    let item = acc.into_response_item();
                    let _ = tx.send(Ok(ResponseEvent::OutputItemDone(item))).await;
                }
                let _ = tx.send(Ok(ResponseEvent::Completed {
                    response_id: response_id.clone(),
                    token_usage: None, // Mapped from Usage event if received
                })).await;
                break;
            }
            ChatCompletionEvent::Error(msg) => {
                let _ = tx.send(Err(ApiError::Api {
                    status_code: 500,
                    message: msg,
                })).await;
                break;
            }
        }
    }
}
```

- [ ] **Step 2: Map Anthropic events to ResponseEvent**

```rust
/// Map Anthropic SSE events to the common ResponseEvent format.
pub(crate) async fn map_anthropic_events(
    mut stream: AnthropicStream,
    tx: mpsc::Sender<Result<ResponseEvent, ApiError>>,
) {
    let mut current_text = String::new();
    let mut current_tool_json = String::new();
    let mut current_tool_name = String::new();
    let mut current_tool_id = String::new();

    while let Some(event) = stream.next().await {
        match event {
            AnthropicEvent::MessageStart(msg) => {
                let _ = tx.send(Ok(ResponseEvent::Created)).await;
            }
            AnthropicEvent::ContentBlockStart(block) => {
                match block.content_block {
                    ContentBlockType::ToolUse { id, name } => {
                        current_tool_id = id;
                        current_tool_name = name;
                        current_tool_json.clear();
                    }
                    _ => {}
                }
            }
            AnthropicEvent::ContentBlockDelta(delta) => {
                match delta.delta {
                    DeltaType::TextDelta { text } => {
                        current_text.push_str(&text);
                        // Emit incremental text as OutputItemAdded
                    }
                    DeltaType::InputJsonDelta { partial_json } => {
                        current_tool_json.push_str(&partial_json);
                    }
                    DeltaType::ThinkingDelta { thinking } => {
                        // Emit as Reasoning item
                    }
                }
            }
            AnthropicEvent::ContentBlockStop { .. } => {
                // If we accumulated tool JSON, emit as FunctionCall
                if !current_tool_json.is_empty() {
                    let item = ResponseItem::LocalShellCall {
                        call_id: Some(current_tool_id.clone()),
                        name: Some(current_tool_name.clone()),
                        arguments: Some(current_tool_json.clone()),
                        ..Default::default()
                    };
                    let _ = tx.send(Ok(ResponseEvent::OutputItemDone(item))).await;
                }
            }
            AnthropicEvent::MessageDelta(delta) => {
                // Extract usage
            }
            AnthropicEvent::MessageStop => {
                let _ = tx.send(Ok(ResponseEvent::Completed {
                    response_id: String::new(),
                    token_usage: None,
                })).await;
                break;
            }
            _ => {}
        }
    }
}
```

- [ ] **Step 3: Map Gemini events to ResponseEvent**

Similar pattern: accumulate text/function call parts from candidates, emit as ResponseItem variants.

- [ ] **Step 4: Commit**

```bash
git add codex-rs/core/src/provider_event_mapper.rs
git commit -m "Create provider event mapper: Chat Completions, Anthropic, Gemini -> ResponseEvent"
```

---

### Task 5: Integrate Router into ModelClient

**Files:**
- Modify: `codex-rs/core/src/client.rs`

- [ ] **Step 1: Add ProviderRouter to ModelClientState**

In `ModelClientState` (line ~133 in `client.rs`), add a `ProviderRouter`:

```rust
struct ModelClientState {
    // ... existing fields ...
    provider_router: ProviderRouter,
}
```

- [ ] **Step 2: Branch on WireProtocol in stream method**

In the `ModelClient` or `ModelClientSession` stream method, add a routing branch:

```rust
match self.state.provider.wire_api {
    WireProtocol::Responses => {
        // Existing code path -- SSE or WebSocket to Responses API
        // This entire existing block is UNCHANGED
        self.stream_responses_api(/* ... */).await
    }
    other => {
        // New path -- delegate to ProviderRouter
        self.state.provider_router.stream(
            &self.state.provider,
            &auth,
            model,
            &prompt.input,
            &prompt.tools,
            &prompt.base_instructions.to_string(),
            extra_headers,
        ).await
    }
}
```

The key principle: the existing Responses API code path must not be touched. All new wire protocols go through the router.

- [ ] **Step 3: Handle auth for non-OpenAI providers**

The current `ModelClient` resolves auth via `AuthManager` which is OpenAI-specific. Add a fallback:

```rust
// In the client setup flow:
let auth = match self.state.provider.wire_api {
    WireProtocol::Responses => {
        // Existing OpenAI auth resolution
        self.resolve_openai_auth().await?
    }
    _ => {
        // Multi-provider auth resolution
        resolve_auth_for_provider(&provider_config, &auth_json)
            .ok_or_else(|| CodexErr::NoAuthForProvider(provider_config.id.clone()))?
    }
};
```

- [ ] **Step 4: Commit**

```bash
git add codex-rs/core/src/client.rs
git commit -m "Integrate ProviderRouter into ModelClient with WireProtocol branching"
```

---

### Task 6: Update Context Manager for Multi-Provider Token Counting

**Files:**
- Modify: `codex-rs/core/src/context_manager/history.rs`

- [ ] **Step 1: Adjust token estimation**

The context manager uses byte-based estimation for token counting (in `estimate_response_item_model_visible_bytes`). Different providers have different tokenizers:
- OpenAI: ~4 chars per token
- Anthropic: ~3.5 chars per token
- Gemini: ~4 chars per token

For now, keep the existing byte-based estimation but add a `tokens_per_byte_estimate` multiplier that can be tuned per provider:

```rust
/// Get the approximate tokens-per-byte ratio for a provider's tokenizer.
pub(crate) fn tokens_per_byte_ratio(wire_protocol: WireProtocol) -> f64 {
    match wire_protocol {
        WireProtocol::Responses => 0.25,       // ~4 bytes/token (OpenAI)
        WireProtocol::ChatCompletions => 0.25,  // Most are OpenAI-compatible
        WireProtocol::Anthropic => 0.29,        // ~3.5 bytes/token (Claude)
        WireProtocol::Gemini => 0.25,           // ~4 bytes/token
    }
}
```

- [ ] **Step 2: Use model context window from ModelDef**

Currently the context window is hardcoded or comes from model metadata JSON. Wire it to use the `ModelDef.context_window` from the provider registry:

```rust
// In context management decisions:
let context_window = model_def
    .map(|m| m.context_window)
    .unwrap_or(128_000); // Fallback for unknown models
```

- [ ] **Step 3: Commit**

```bash
git add codex-rs/core/src/context_manager/
git commit -m "Update context manager with per-provider token estimation and model context windows"
```

---

### Task 7: Wire Provider Registry into Config Loading

**Files:**
- Modify: `codex-rs/core/src/config/mod.rs`
- Modify: `codex-rs/core/src/model_provider_info.rs`

- [ ] **Step 1: Merge built-in providers from orbit-providers registry**

In `built_in_model_providers()` (in `model_provider_info.rs`), merge the existing OpenAI/Ollama/LM Studio providers with the full registry from `orbit-providers`:

```rust
pub fn built_in_model_providers(
    openai_base_url: Option<String>,
) -> HashMap<String, ModelProviderInfo> {
    let mut providers = HashMap::new();

    // Existing built-in providers
    let openai = ModelProviderInfo::create_openai_provider(openai_base_url);
    providers.insert(OPENAI_PROVIDER_ID.to_string(), openai);

    // Load from orbit-providers registry
    for (id, config) in orbit_providers::registry::built_in_providers() {
        if !providers.contains_key(&id) {
            providers.insert(id, config.into());
        }
    }

    providers
}
```

- [ ] **Step 2: Add `From<ProviderConfig>` for `ModelProviderInfo`**

Create a conversion from the new `orbit-providers::ProviderConfig` to the existing `ModelProviderInfo` so both systems interoperate during the transition:

```rust
impl From<orbit_providers::ProviderConfig> for ModelProviderInfo {
    fn from(config: orbit_providers::ProviderConfig) -> Self {
        ModelProviderInfo {
            name: config.name,
            base_url: Some(config.base_url),
            env_key: config.env_keys.first().cloned(),
            env_key_instructions: config.env_key_instructions,
            wire_api: config.wire_protocol,
            // ... map other fields
        }
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add codex-rs/core/src/config/mod.rs codex-rs/core/src/model_provider_info.rs
git commit -m "Wire orbit-providers registry into config loading and provider resolution"
```

---

### Task 8: Add Module Declarations and Final Verification

- [ ] **Step 1: Add module declarations in core/src/lib.rs**

```rust
mod provider_event_mapper;
mod provider_router;
```

- [ ] **Step 2: Full workspace build**

```bash
cd codex-rs && cargo build
```

Expected: Clean build.

- [ ] **Step 3: Run core tests**

```bash
cd codex-rs && cargo test -p orbit-code-core 2>&1 | tail -30
```

Expected: All existing tests pass. New wire protocol paths may not have integration tests yet (those come with end-to-end testing).

- [ ] **Step 4: Verify Responses API path is unchanged**

```bash
# Run a specific test that exercises the Responses API path
cd codex-rs && cargo test -p orbit-code-core -- client
```

- [ ] **Step 5: Commit and push**

```bash
git add -A
git commit -m "Stage 7 complete: core integration with multi-provider routing, event mapping, and context management"
git push origin main
```

---

## Expected Outcomes

After Stage 7:
- `ModelClient` can route requests to 4 different wire protocols based on provider config
- The existing Responses API code path is completely unchanged -- zero regression risk for OpenAI
- `ProviderRouter` dispatches to `orbit-chat-completions`, `orbit-anthropic`, or `orbit-gemini` based on `WireProtocol`
- Provider-specific events are mapped back to the common `ResponseEvent` format
- The rest of the core engine (agent loop, tool execution, context management, rollout recording) works identically regardless of provider
- Context manager uses per-provider token estimation ratios
- Provider registry is merged into config loading, so user-defined providers in `config.toml` work alongside built-in providers
- ~1,000 lines of new code in core (router, event mapper, config updates)
