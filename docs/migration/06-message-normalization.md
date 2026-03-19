# Stage 6: Message Normalization

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the per-provider message transformation quirks from the TypeScript `transform.ts` into a Rust `MessageNormalizer` trait and per-provider implementations, ensuring each provider receives correctly formatted messages, tool IDs, cache hints, and model-specific parameters.

**Architecture:** The core engine works with `ResponseItem` (the Responses API format from `orbit-code-protocol`). Before sending to a non-Responses provider, messages must be transformed into the provider's expected format. This stage creates a normalization layer in `orbit-providers` that converts `ResponseItem` sequences into the wire-protocol-specific message types defined in the Stage 3 client crates. Each provider can have quirks (Anthropic tool ID sanitization, Mistral tool ID padding, Google content structure, etc.) that are encapsulated in provider-specific normalizers.

**Tech Stack:** Rust, `serde_json`, `regex`.

**Depends on:** Stage 3 (client crate types exist), Stage 4 (provider registry with `WireProtocol`).

---

## Reference: TypeScript Transform Quirks

The TypeScript `provider/transform.ts` contains these provider-specific behaviors:

### Anthropic
- **Empty content filtering:** Reject messages with empty string content; filter out empty text/reasoning parts from array content.
- **Tool ID sanitization:** Replace non-`[a-zA-Z0-9_-]` characters in `toolCallId` with `_` for Claude models.
- **Cache control:** Apply `{ anthropic: { cacheControl: { type: "ephemeral" } } }` to first 2 system messages and last 2 conversation messages.
- **Anthropic beta headers:** `anthropic-beta: claude-code-20250219,interleaved-thinking-2025-05-14,fine-grained-tool-streaming-2025-05-14`.

### Mistral
- **Tool ID normalization:** Replace non-alphanumeric characters, take first 9 chars, pad to exactly 9 chars with zeros.
- **Dummy assistant messages:** Insert `{ role: "assistant", content: [{ type: "text", text: "Done." }] }` between `tool` and `user` messages (Mistral rejects tool->user sequences).

### Google Gemini
- **Different content structure:** Uses `parts` instead of `content`, `model` role instead of `assistant`, `functionCall`/`functionResponse` instead of tool_call/tool_result.
- **No tool_call_id matching:** Gemini does not use tool call IDs in the same way.

### OpenAI / OpenAI-Compatible
- **Reasoning content:** Models with `interleaved` capability use a provider-specific field name (`reasoning_content` for DeepSeek, `reasoning_details` for some others) to pass reasoning text.
- **Provider options remapping:** The `providerOptions` key may differ between stored provider ID and SDK-expected key.

### Temperature/TopP/TopK Defaults
- Qwen: temperature=0.55, topP=1
- Claude: temperature=undefined (let provider default)
- Gemini: temperature=1.0, topK=64
- GLM: temperature=1.0
- Kimi: temperature varies by sub-model
- MiniMax: temperature=1.0, topP=0.95, topK=20-40

### Unsupported Modalities
- Check model capabilities before sending image/audio/video/PDF parts
- Replace unsupported parts with error text messages

---

## Files Created

**In `orbit-providers` crate:**
- `codex-rs/orbit-providers/src/normalize.rs` -- `MessageNormalizer` trait and shared utilities
- `codex-rs/orbit-providers/src/normalize/anthropic.rs` -- Anthropic-specific transforms
- `codex-rs/orbit-providers/src/normalize/mistral.rs` -- Mistral-specific transforms
- `codex-rs/orbit-providers/src/normalize/gemini.rs` -- Gemini content conversion
- `codex-rs/orbit-providers/src/normalize/chat_completions.rs` -- Generic Chat Completions conversion
- `codex-rs/orbit-providers/src/normalize/reasoning.rs` -- Reasoning content handling
- `codex-rs/orbit-providers/src/normalize/modality.rs` -- Unsupported modality filtering

**Modified:**
- `codex-rs/orbit-providers/Cargo.toml` -- Add dependencies on client crate types
- `codex-rs/orbit-providers/src/lib.rs` -- Add normalize module

---

### Task 1: Define the `MessageNormalizer` Trait

**Files:**
- Create: `codex-rs/orbit-providers/src/normalize.rs` (module root)

- [ ] **Step 1: Define the trait**

```rust
pub mod anthropic;
pub mod chat_completions;
pub mod gemini;
pub mod mistral;
pub mod modality;
pub mod reasoning;

use crate::model_def::ModelDef;
use crate::provider_config::ProviderConfig;

/// Converts the internal ResponseItem-based conversation history into
/// wire-protocol-specific message formats for a given provider.
pub trait MessageNormalizer: Send + Sync {
    /// Convert a sequence of ResponseItems into the provider's message format.
    fn normalize_messages(
        &self,
        items: &[orbit_code_protocol::models::ResponseItem],
        model: &ModelDef,
        provider: &ProviderConfig,
    ) -> NormalizedMessages;

    /// Get model-specific generation parameters.
    fn generation_params(&self, model: &ModelDef) -> GenerationParams;
}

/// The result of normalizing messages for a specific wire protocol.
pub enum NormalizedMessages {
    /// For Chat Completions API providers.
    ChatCompletions(Vec<orbit_chat_completions::ChatMessage>),
    /// For Anthropic Messages API.
    Anthropic {
        messages: Vec<orbit_anthropic::AnthropicMessage>,
        system: Vec<orbit_anthropic::SystemBlock>,
    },
    /// For Google Gemini GenerateContent API.
    Gemini {
        contents: Vec<orbit_gemini::GeminiContent>,
        system_instruction: Option<orbit_gemini::GeminiContent>,
    },
    /// For OpenAI Responses API (pass through -- no conversion needed).
    Responses(Vec<orbit_code_protocol::models::ResponseItem>),
}

/// Model-specific generation parameters resolved from the model definition
/// and provider quirks.
#[derive(Debug, Clone, Default)]
pub struct GenerationParams {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u64>,
    pub max_output_tokens: Option<u64>,
    pub reasoning_effort: Option<String>,
}
```

- [ ] **Step 2: Implement factory function**

```rust
/// Create the appropriate normalizer for a given wire protocol.
pub fn normalizer_for(protocol: WireProtocol, provider: &ProviderConfig) -> Box<dyn MessageNormalizer> {
    match protocol {
        WireProtocol::Responses => Box::new(ResponsesNormalizer),
        WireProtocol::ChatCompletions => {
            if is_mistral_provider(provider) {
                Box::new(mistral::MistralNormalizer)
            } else {
                Box::new(chat_completions::ChatCompletionsNormalizer)
            }
        }
        WireProtocol::Anthropic => Box::new(anthropic::AnthropicNormalizer),
        WireProtocol::Gemini => Box::new(gemini::GeminiNormalizer),
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add codex-rs/orbit-providers/src/normalize.rs
git commit -m "Define MessageNormalizer trait and NormalizedMessages enum"
```

---

### Task 2: Implement Chat Completions Normalizer

**Files:**
- Create: `codex-rs/orbit-providers/src/normalize/chat_completions.rs`

- [ ] **Step 1: Convert ResponseItem to ChatMessage**

Map the internal `ResponseItem` variants to `ChatMessage`:

```rust
pub struct ChatCompletionsNormalizer;

impl ChatCompletionsNormalizer {
    /// Convert a single ResponseItem into zero or more ChatMessages.
    fn convert_item(item: &ResponseItem) -> Vec<ChatMessage> {
        match item {
            ResponseItem::Message { role, content, .. } => {
                // Map "system" / "user" / "assistant" roles
                // Convert ContentItem::Text, ContentItem::Image, etc.
                vec![ChatMessage { role: map_role(role), content: convert_content(content), .. }]
            }
            ResponseItem::LocalShellCall { call_id, name, arguments, .. } => {
                // Convert to assistant message with tool_calls
                vec![ChatMessage {
                    role: MessageRole::Assistant,
                    tool_calls: Some(vec![ToolCall {
                        id: call_id.clone(),
                        r#type: "function".to_string(),
                        function: FunctionCall { name: name.clone(), arguments: arguments.clone() },
                    }]),
                    ..
                }]
            }
            ResponseItem::FunctionCallOutput { call_id, output, .. } => {
                // Convert to tool message
                vec![ChatMessage {
                    role: MessageRole::Tool,
                    tool_call_id: Some(call_id.clone()),
                    content: ChatContent::Text(output.clone()),
                    ..
                }]
            }
            // Handle other ResponseItem variants...
            _ => vec![],
        }
    }
}
```

- [ ] **Step 2: Implement system message extraction**

Extract system instructions from `BaseInstructions` and prepend as a system message.

- [ ] **Step 3: Handle reasoning content for OpenAI-compatible providers**

For models with `interleaved` capability, include reasoning in the configured field:

```rust
// Reference: ProviderTransform in transform.ts
// If model has interleaved config, store reasoning in providerOptions
fn handle_reasoning(msg: &mut ChatMessage, model: &ModelDef, reasoning_text: &str) {
    if let Some(interleaved) = &model.capabilities.interleaved {
        // Add reasoning_content / reasoning_details field
        // This is passed as an extra JSON field in the message
    }
}
```

- [ ] **Step 4: Implement generation params**

Port temperature/topP/topK defaults from `transform.ts`:

```rust
impl MessageNormalizer for ChatCompletionsNormalizer {
    fn generation_params(&self, model: &ModelDef) -> GenerationParams {
        GenerationParams {
            temperature: model.default_temperature,
            top_p: model.default_top_p,
            top_k: model.default_top_k,
            max_output_tokens: Some(model.default_max_output_tokens),
            ..Default::default()
        }
    }
}
```

- [ ] **Step 5: Commit**

```bash
git add codex-rs/orbit-providers/src/normalize/chat_completions.rs
git commit -m "Implement Chat Completions message normalizer with ResponseItem conversion"
```

---

### Task 3: Implement Anthropic Normalizer

**Files:**
- Create: `codex-rs/orbit-providers/src/normalize/anthropic.rs`

- [ ] **Step 1: Convert ResponseItem to AnthropicMessage**

Anthropic uses a different message structure:
- System messages go in a separate `system` field (not in the messages array)
- Assistant messages with tool calls use `tool_use` content blocks
- Tool results are `tool_result` content blocks in user messages

```rust
pub struct AnthropicNormalizer;

impl AnthropicNormalizer {
    fn convert_items(items: &[ResponseItem]) -> (Vec<AnthropicMessage>, Vec<SystemBlock>) {
        let mut messages = Vec::new();
        let mut system_blocks = Vec::new();

        for item in items {
            match item {
                ResponseItem::Message { role, content, .. } if role == "system" => {
                    system_blocks.push(SystemBlock {
                        r#type: "text".to_string(),
                        text: extract_text(content),
                        cache_control: None,
                    });
                }
                // ... convert other items
            }
        }

        (messages, system_blocks)
    }
}
```

- [ ] **Step 2: Implement tool ID sanitization**

Port from `transform.ts`: replace `[^a-zA-Z0-9_-]` with `_` in tool call IDs for Claude models.

```rust
/// Sanitize tool call IDs for Anthropic.
/// Claude rejects IDs with characters outside [a-zA-Z0-9_-].
fn sanitize_tool_id(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect()
}
```

- [ ] **Step 3: Implement empty content filtering**

Port from `transform.ts`: filter out messages with empty content and remove empty text/reasoning parts.

```rust
/// Filter out empty content blocks that Anthropic rejects.
fn filter_empty_content(messages: &mut Vec<AnthropicMessage>) {
    messages.retain(|msg| {
        match &msg.content {
            AnthropicContent::Text(s) => !s.is_empty(),
            AnthropicContent::Blocks(blocks) => {
                !blocks.is_empty() // Also filter empty text blocks within
            }
        }
    });
}
```

- [ ] **Step 4: Implement cache control hints**

Apply `cache_control: { type: "ephemeral" }` to:
- First 2 system blocks
- Last 2 conversation messages

```rust
/// Apply Anthropic cache control hints for prompt caching.
fn apply_cache_control(system: &mut [SystemBlock], messages: &mut [AnthropicMessage]) {
    let cache = Some(CacheControl { r#type: "ephemeral".to_string() });

    // Mark first 2 system blocks
    for block in system.iter_mut().take(2) {
        block.cache_control = cache.clone();
    }

    // Mark last 2 non-system messages
    let len = messages.len();
    for msg in messages.iter_mut().rev().take(2) {
        // Apply cache control to the message or its last content block
    }
}
```

- [ ] **Step 5: Commit**

```bash
git add codex-rs/orbit-providers/src/normalize/anthropic.rs
git commit -m "Implement Anthropic message normalizer with tool ID sanitization and cache control"
```

---

### Task 4: Implement Mistral Normalizer

**Files:**
- Create: `codex-rs/orbit-providers/src/normalize/mistral.rs`

- [ ] **Step 1: Extend ChatCompletionsNormalizer with Mistral quirks**

```rust
pub struct MistralNormalizer;
```

- [ ] **Step 2: Implement tool ID padding**

Port from `transform.ts`: Mistral requires alphanumeric tool call IDs with exactly 9 characters.

```rust
/// Normalize tool call IDs for Mistral.
/// Requirements: alphanumeric only, exactly 9 characters.
fn normalize_mistral_tool_id(id: &str) -> String {
    id.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(9)
        .collect::<String>()
        .chars()
        .chain(std::iter::repeat('0'))
        .take(9)
        .collect()
}
```

- [ ] **Step 3: Implement dummy assistant message insertion**

Port from `transform.ts`: insert `{ role: "assistant", content: "Done." }` between tool and user messages.

```rust
/// Mistral rejects message sequences where a tool message is followed
/// by a user message. Insert a dummy assistant message between them.
fn insert_dummy_assistant_messages(messages: &mut Vec<ChatMessage>) {
    let mut i = 0;
    while i + 1 < messages.len() {
        if messages[i].role == MessageRole::Tool
            && messages[i + 1].role == MessageRole::User
        {
            messages.insert(i + 1, ChatMessage {
                role: MessageRole::Assistant,
                content: ChatContent::Text("Done.".to_string()),
                ..Default::default()
            });
            i += 2; // Skip the inserted message
        } else {
            i += 1;
        }
    }
}
```

- [ ] **Step 4: Commit**

```bash
git add codex-rs/orbit-providers/src/normalize/mistral.rs
git commit -m "Implement Mistral normalizer with tool ID padding and dummy assistant messages"
```

---

### Task 5: Implement Gemini Normalizer

**Files:**
- Create: `codex-rs/orbit-providers/src/normalize/gemini.rs`

- [ ] **Step 1: Convert ResponseItem to GeminiContent**

Gemini uses a fundamentally different message structure:
- Role is `"user"` or `"model"` (not `"assistant"`)
- Content uses `parts` array (not `content`)
- Tool calls are `functionCall` parts
- Tool results are `functionResponse` parts

```rust
pub struct GeminiNormalizer;

impl GeminiNormalizer {
    fn convert_item(item: &ResponseItem) -> Option<GeminiContent> {
        match item {
            ResponseItem::Message { role, content, .. } => {
                let gemini_role = match role.as_str() {
                    "assistant" => "model",
                    "system" => return None, // System goes in system_instruction
                    other => other,
                };
                Some(GeminiContent {
                    role: Some(gemini_role.to_string()),
                    parts: convert_parts(content),
                })
            }
            ResponseItem::LocalShellCall { name, arguments, .. } => {
                Some(GeminiContent {
                    role: Some("model".to_string()),
                    parts: vec![GeminiPart::FunctionCall {
                        function_call: GeminiFunctionCall {
                            name: name.clone(),
                            args: serde_json::from_str(arguments).unwrap_or_default(),
                        },
                    }],
                })
            }
            ResponseItem::FunctionCallOutput { name, output, .. } => {
                Some(GeminiContent {
                    role: Some("user".to_string()),
                    parts: vec![GeminiPart::FunctionResponse {
                        function_response: GeminiFunctionResponse {
                            name: name.clone(),
                            response: serde_json::json!({ "result": output }),
                        },
                    }],
                })
            }
            _ => None,
        }
    }
}
```

- [ ] **Step 2: Extract system instruction**

System messages go into a separate `system_instruction` field:

```rust
fn extract_system_instruction(items: &[ResponseItem]) -> Option<GeminiContent> {
    let system_parts: Vec<GeminiPart> = items
        .iter()
        .filter_map(|item| match item {
            ResponseItem::Message { role, content, .. } if role == "system" => {
                Some(GeminiPart::Text { text: extract_text(content) })
            }
            _ => None,
        })
        .collect();

    if system_parts.is_empty() {
        None
    } else {
        Some(GeminiContent { role: None, parts: system_parts })
    }
}
```

- [ ] **Step 3: Commit**

```bash
git add codex-rs/orbit-providers/src/normalize/gemini.rs
git commit -m "Implement Gemini message normalizer with content/part conversion"
```

---

### Task 6: Implement Unsupported Modality Filtering

**Files:**
- Create: `codex-rs/orbit-providers/src/normalize/modality.rs`

- [ ] **Step 1: Port unsupported parts logic from transform.ts**

Before sending messages, check model capabilities and replace unsupported content parts with error text:

```rust
/// Check if a content part is supported by the model and replace
/// unsupported parts with an error message.
pub fn filter_unsupported_modalities(
    items: &mut [ResponseItem],
    model: &ModelDef,
) {
    for item in items.iter_mut() {
        if let ResponseItem::Message { content, .. } = item {
            for part in content.iter_mut() {
                match part {
                    ContentItem::Image { .. } if !model.capabilities.input.image => {
                        *part = ContentItem::Text {
                            text: "ERROR: Cannot read image (this model does not support image input). Inform the user.".to_string(),
                        };
                    }
                    // Handle audio, video, PDF similarly
                    _ => {}
                }
            }
        }
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add codex-rs/orbit-providers/src/normalize/modality.rs
git commit -m "Add unsupported modality filtering for multi-provider messages"
```

---

### Task 7: Add Tests and Final Verification

- [ ] **Step 1: Test Anthropic tool ID sanitization**

```rust
#[test]
fn test_sanitize_tool_id() {
    assert_eq!(sanitize_tool_id("call_abc123"), "call_abc123");
    assert_eq!(sanitize_tool_id("call/abc.123"), "call_abc_123");
    assert_eq!(sanitize_tool_id("toolu_01A"), "toolu_01A");
}
```

- [ ] **Step 2: Test Mistral tool ID padding**

```rust
#[test]
fn test_normalize_mistral_tool_id() {
    assert_eq!(normalize_mistral_tool_id("abc123def"), "abc123def");
    assert_eq!(normalize_mistral_tool_id("ab"), "ab0000000");
    assert_eq!(normalize_mistral_tool_id("abcdefghijklmnop"), "abcdefghi");
    assert_eq!(normalize_mistral_tool_id("abc-123_def"), "abc123def");
}
```

- [ ] **Step 3: Test Mistral dummy assistant insertion**

```rust
#[test]
fn test_mistral_dummy_assistant() {
    let mut msgs = vec![
        ChatMessage { role: MessageRole::Tool, .. },
        ChatMessage { role: MessageRole::User, .. },
    ];
    insert_dummy_assistant_messages(&mut msgs);
    assert_eq!(msgs.len(), 3);
    assert_eq!(msgs[1].role, MessageRole::Assistant);
}
```

- [ ] **Step 4: Test Gemini role mapping**

```rust
#[test]
fn test_gemini_role_mapping() {
    // "assistant" -> "model", "user" -> "user", "system" -> separate
}
```

- [ ] **Step 5: Run all tests**

```bash
cd codex-rs && cargo test -p orbit-providers
```

- [ ] **Step 6: Commit and push**

```bash
git add -A
git commit -m "Stage 6 complete: message normalization with per-provider quirks"
git push origin main
```

---

## Expected Outcomes

After Stage 6:
- `MessageNormalizer` trait provides a clean abstraction for per-provider message conversion
- Anthropic normalizer handles: empty content filtering, tool ID sanitization, cache control hints
- Mistral normalizer handles: 9-char alphanumeric tool IDs, dummy assistant message insertion
- Gemini normalizer handles: `user`/`model` roles, `parts` structure, `functionCall`/`functionResponse`
- Chat Completions normalizer handles: generic ResponseItem-to-ChatMessage conversion with reasoning content
- Unsupported modality filtering replaces unsupported content parts with error text
- Generation params (temperature, topP, topK) are resolved per model definition
- All transforms have unit tests with edge cases
- ~1,200 lines of new code in `orbit-providers/src/normalize/`
