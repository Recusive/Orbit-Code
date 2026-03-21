//! Bridge between Codex core types and the Anthropic Messages API.

use crate::Prompt;
use crate::ResponseEvent;
use crate::ResponseStream;
use crate::anthropic_auth::strip_oauth_tool_prefix;
use crate::client_common::tools::FreeformTool;
use crate::client_common::tools::ResponsesApiTool;
use crate::client_common::tools::ToolSpec;
use crate::error::CodexErr;
use crate::error::Result;
use crate::tools::spec::JsonSchema;
use futures::Stream;
use futures::StreamExt;
use http::HeaderMap;
use http::HeaderValue;
use orbit_code_anthropic::ANTHROPIC_BETA_HEADER_VALUE;
use orbit_code_anthropic::AnthropicError;
use orbit_code_anthropic::AnthropicEvent;
use orbit_code_anthropic::CONTEXT_1M_BETA_HEADER_VALUE;
use orbit_code_anthropic::Content;
use orbit_code_anthropic::ContentBlock;
use orbit_code_anthropic::ImageSource;
use orbit_code_anthropic::Message;
use orbit_code_anthropic::MessagesRequest;
use orbit_code_anthropic::OutputConfig;
use orbit_code_anthropic::Role;
use orbit_code_anthropic::SystemBlock;
use orbit_code_anthropic::ThinkingConfig;
use orbit_code_anthropic::Tool;
use orbit_code_anthropic::ToolChoice;
use orbit_code_protocol::models::ContentItem;
use orbit_code_protocol::models::ResponseItem;
use orbit_code_protocol::openai_models::ReasoningEffort as ReasoningEffortConfig;
use orbit_code_protocol::protocol::TokenUsage;
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::mpsc;

pub(crate) const ANTHROPIC_EXPLICIT_MODEL_REQUIRED_ERROR: &str = "Anthropic stage 3a requires an explicit Claude model when `model_provider = \"anthropic\"` is set.";
pub(crate) const CLAUDE_PROVIDER_MISMATCH_ERROR: &str =
    "Claude models require `model_provider = \"anthropic\"` in stage 3a.";
const ANTHROPIC_UNSUPPORTED_MODEL_ERROR_PREFIX: &str =
    "Anthropic stage 3a supports Claude models only; received";
const ANTHROPIC_1M_HEADER_ERROR_PREFIX: &str =
    "Anthropic stage 3a failed to add required beta header";
const DEFAULT_ANTHROPIC_MAX_TOKENS: u64 = 32_000;
const RESPONSE_STREAM_CHANNEL_CAPACITY: usize = 128;

pub(crate) struct AnthropicModelDefaults {
    pub(crate) max_tokens: u64,
    pub(crate) thinking: Option<ThinkingConfig>,
    pub(crate) additional_beta_headers: Vec<&'static str>,
    pub(crate) effort: Option<String>,
}

pub fn is_known_anthropic_model(model: &str) -> bool {
    model.starts_with("claude-")
}

pub(crate) fn anthropic_model_defaults(
    slug: &str,
    effort: Option<ReasoningEffortConfig>,
) -> Result<AnthropicModelDefaults> {
    if !is_known_anthropic_model(slug) {
        return Err(CodexErr::InvalidRequest(format!(
            "{ANTHROPIC_UNSUPPORTED_MODEL_ERROR_PREFIX} `{slug}`."
        )));
    }

    let normalized_effort = effort.unwrap_or(ReasoningEffortConfig::Medium);
    // Opus 4.6 and Sonnet 4.6 use adaptive thinking.
    // Other models use budgeted thinking.
    let thinking = if uses_adaptive_thinking(slug) {
        Some(ThinkingConfig::Adaptive {})
    } else {
        budgeted_thinking_config(DEFAULT_ANTHROPIC_MAX_TOKENS, normalized_effort)
    };

    let additional_beta_headers = if requires_1m_context(slug) {
        vec![CONTEXT_1M_BETA_HEADER_VALUE]
    } else {
        Vec::new()
    };

    // effort parameter is only supported on Opus 4.6, Sonnet 4.6, and Opus 4.5.
    let effort = if supports_effort_parameter(slug) {
        Some(map_reasoning_effort_to_anthropic(normalized_effort))
    } else {
        None
    };

    Ok(AnthropicModelDefaults {
        max_tokens: DEFAULT_ANTHROPIC_MAX_TOKENS,
        thinking,
        additional_beta_headers,
        effort,
    })
}

fn map_reasoning_effort_to_anthropic(effort: ReasoningEffortConfig) -> String {
    match effort {
        ReasoningEffortConfig::None
        | ReasoningEffortConfig::Minimal
        | ReasoningEffortConfig::Low => "low".to_string(),
        ReasoningEffortConfig::Medium => "medium".to_string(),
        ReasoningEffortConfig::High => "high".to_string(),
        ReasoningEffortConfig::XHigh => "max".to_string(),
    }
}

pub(crate) fn build_messages_request(
    prompt: &Prompt,
    model: &str,
    defaults: &AnthropicModelDefaults,
) -> Result<MessagesRequest> {
    let anthropic_tools = tools_to_anthropic_format(&prompt.tools)?;
    let mut system = vec![SystemBlock {
        r#type: "text".to_string(),
        text: prompt.base_instructions.text.clone(),
        cache_control: None,
    }];
    if let Some(schema) = &prompt.output_schema {
        let schema = serde_json::to_string(schema).map_err(CodexErr::from)?;
        system.push(SystemBlock {
            r#type: "text".to_string(),
            text: format!("Return JSON that matches this schema exactly: {schema}"),
            cache_control: None,
        });
    }

    let mut messages = translate_input_to_messages(&prompt.get_formatted_input(), &mut system)?;
    sanitize_messages(&mut messages);
    if messages.is_empty() {
        return Err(CodexErr::InvalidRequest(
            "Anthropic stage 3a requires at least one non-empty message after sanitization."
                .to_string(),
        ));
    }

    Ok(MessagesRequest {
        model: model.to_string(),
        messages,
        system: (!system.is_empty()).then_some(system),
        tools: (!anthropic_tools.is_empty()).then_some(anthropic_tools),
        tool_choice: (!prompt.tools.is_empty()).then_some(ToolChoice::Auto),
        thinking: defaults.thinking.clone(),
        max_tokens: defaults.max_tokens,
        stream: true,
        temperature: None,
        top_p: None,
        top_k: None,
        metadata: None,
        output_config: defaults
            .effort
            .as_ref()
            .map(|e| OutputConfig { effort: e.clone() }),
    })
}

pub(crate) fn tools_to_anthropic_format(tools: &[ToolSpec]) -> Result<Vec<Tool>> {
    tools
        .iter()
        .map(|tool| match tool {
            ToolSpec::Function(tool) => function_tool_to_anthropic_tool(tool).map(Some),
            ToolSpec::ToolSearch {
                description,
                parameters,
                ..
            } => Ok(Some(Tool {
                name: "tool_search".to_string(),
                description: description.clone(),
                input_schema: serde_json::to_value(parameters).map_err(CodexErr::from)?,
            })),
            ToolSpec::LocalShell {} => Ok(Some(Tool {
                name: "local_shell".to_string(),
                description: "Runs a shell command and returns its output.".to_string(),
                input_schema: serde_json::to_value(local_shell_input_schema())
                    .map_err(CodexErr::from)?,
            })),
            ToolSpec::Freeform(tool) => Ok(Some(freeform_tool_to_anthropic_tool(tool))),
            ToolSpec::ImageGeneration { .. } | ToolSpec::WebSearch { .. } => Ok(None),
        })
        .filter_map(|result| match result {
            Ok(Some(tool)) => Some(Ok(tool)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        })
        .collect()
}

fn function_tool_to_anthropic_tool(tool: &ResponsesApiTool) -> Result<Tool> {
    Ok(Tool {
        name: tool.name.clone(),
        description: tool.description.clone(),
        input_schema: serde_json::to_value(&tool.parameters).map_err(CodexErr::from)?,
    })
}

fn freeform_tool_to_anthropic_tool(tool: &FreeformTool) -> Tool {
    Tool {
        name: tool.name.clone(),
        description: tool.description.clone(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "The raw freeform input for this tool."
                }
            },
            "required": ["input"],
            "additionalProperties": false
        }),
    }
}

fn local_shell_input_schema() -> JsonSchema {
    JsonSchema::Object {
        properties: std::collections::BTreeMap::from([
            (
                "command".to_string(),
                JsonSchema::Array {
                    items: Box::new(JsonSchema::String { description: None }),
                    description: Some("The command to execute".to_string()),
                },
            ),
            (
                "workdir".to_string(),
                JsonSchema::String {
                    description: Some(
                        "The working directory to execute the command in".to_string(),
                    ),
                },
            ),
            (
                "timeout_ms".to_string(),
                JsonSchema::Number {
                    description: Some("The timeout for the command in milliseconds".to_string()),
                },
            ),
        ]),
        required: Some(vec!["command".to_string()]),
        additional_properties: Some(false.into()),
    }
}

pub(crate) fn sanitize_messages(messages: &mut Vec<Message>) {
    messages.retain(|message| match &message.content {
        Content::Text(text) => !text.trim().is_empty(),
        Content::Blocks(blocks) => !blocks.is_empty(),
    });

    for message in messages.iter_mut() {
        if let Content::Blocks(blocks) = &mut message.content {
            blocks.retain(|block| match block {
                ContentBlock::Text { text, .. } => !text.trim().is_empty(),
                _ => true,
            });
        }
    }

    messages.retain(|message| match &message.content {
        Content::Text(text) => !text.trim().is_empty(),
        Content::Blocks(blocks) => !blocks.is_empty(),
    });
}

pub(crate) fn sanitize_tool_call_id(id: &str) -> String {
    id.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' || character == '-' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

pub(crate) fn merge_anthropic_beta_headers(
    headers: &mut HeaderMap,
    model: &str,
    additional_beta_headers: &[&'static str],
) -> Result<()> {
    let mut beta_headers = headers
        .get("anthropic-beta")
        .and_then(|value| value.to_str().ok())
        .map(parse_beta_header)
        .unwrap_or_else(|| parse_beta_header(ANTHROPIC_BETA_HEADER_VALUE));

    for header in additional_beta_headers {
        if !beta_headers.iter().any(|value| value == header) {
            beta_headers.push((*header).to_string());
        }
    }

    let joined = beta_headers.join(",");
    headers.insert(
        "anthropic-beta",
        HeaderValue::from_str(&joined).map_err(|err| {
            CodexErr::InvalidRequest(format!("invalid Anthropic beta header: {err}"))
        })?,
    );

    for required in additional_beta_headers {
        if !beta_headers.iter().any(|value| value == required) {
            return Err(CodexErr::InvalidRequest(format!(
                "{ANTHROPIC_1M_HEADER_ERROR_PREFIX} `{required}` for model `{model}`."
            )));
        }
    }

    Ok(())
}

pub(crate) fn map_anthropic_error(error: AnthropicError) -> CodexErr {
    match error {
        AnthropicError::RateLimited | AnthropicError::Overloaded => {
            CodexErr::Stream(error.to_string(), None)
        }
        AnthropicError::Api(error) => {
            if (400..500).contains(&error.status) {
                CodexErr::InvalidRequest(format!("[{}] {}", error.error_type, error.message))
            } else {
                CodexErr::Stream(format!("[{}] {}", error.error_type, error.message), None)
            }
        }
        AnthropicError::Transport(transport_error) => match transport_error {
            orbit_code_client::TransportError::Http {
                status,
                body,
                url: _,
                headers: _,
            } if status.is_client_error() => {
                CodexErr::InvalidRequest(body.unwrap_or_else(|| status.to_string()))
            }
            orbit_code_client::TransportError::Http { status, body, .. } => {
                CodexErr::Stream(body.unwrap_or_else(|| status.to_string()), None)
            }
            orbit_code_client::TransportError::RetryLimit => {
                CodexErr::Stream("retry limit reached".to_string(), None)
            }
            orbit_code_client::TransportError::Timeout => CodexErr::Timeout,
            orbit_code_client::TransportError::Network(message)
            | orbit_code_client::TransportError::Build(message) => CodexErr::Stream(message, None),
        },
        AnthropicError::Json(error) => CodexErr::Stream(error.to_string(), None),
        AnthropicError::StreamParse(error) => CodexErr::Stream(error, None),
    }
}

pub(crate) fn map_anthropic_to_response_stream<S>(stream: S, is_oauth: bool) -> ResponseStream
where
    S: Stream<Item = std::result::Result<AnthropicEvent, AnthropicError>> + Send + 'static,
{
    let (tx_event, rx_event) = mpsc::channel(RESPONSE_STREAM_CHANNEL_CAPACITY);
    tokio::spawn(async move {
        let mut stream = Box::pin(stream);
        let mut response_id = String::new();
        let mut input_tokens = 0_u64;
        let mut output_tokens = 0_u64;
        let mut active_blocks: HashMap<u32, ActiveBlock> = HashMap::new();
        let mut message_item_emitted = false;

        while let Some(event) = stream.next().await {
            match event {
                Ok(AnthropicEvent::MessageStart {
                    message_id,
                    model: _,
                    usage,
                }) => {
                    response_id = message_id;
                    input_tokens = usage.input_tokens;
                    output_tokens = usage.output_tokens;
                    if tx_event.send(Ok(ResponseEvent::Created)).await.is_err() {
                        return;
                    }
                }
                Ok(AnthropicEvent::ContentBlockStart { index, block }) => {
                    // Emit OutputItemAdded once before the first content block
                    // that will produce deltas. The core session event loop
                    // requires an active_item before OutputTextDelta or
                    // ReasoningContentDelta events.
                    if !message_item_emitted
                        && matches!(
                            block,
                            orbit_code_anthropic::ContentBlockType::Text { .. }
                                | orbit_code_anthropic::ContentBlockType::Thinking { .. }
                        )
                    {
                        let skeleton = ResponseItem::Message {
                            id: None,
                            role: "assistant".to_string(),
                            content: vec![],
                            end_turn: None,
                            phase: None,
                        };
                        if tx_event
                            .send(Ok(ResponseEvent::OutputItemAdded(skeleton)))
                            .await
                            .is_err()
                        {
                            return;
                        }
                        message_item_emitted = true;
                    }

                    match block {
                        orbit_code_anthropic::ContentBlockType::Text { text } => {
                            if !text.is_empty()
                                && tx_event
                                    .send(Ok(ResponseEvent::OutputTextDelta(text.clone())))
                                    .await
                                    .is_err()
                            {
                                return;
                            }
                            active_blocks.insert(index, ActiveBlock::Text { text });
                        }
                        orbit_code_anthropic::ContentBlockType::ToolUse { id, name } => {
                            active_blocks.insert(
                                index,
                                ActiveBlock::ToolUse {
                                    id,
                                    name,
                                    arguments: String::new(),
                                },
                            );
                        }
                        orbit_code_anthropic::ContentBlockType::Thinking { thinking } => {
                            if !thinking.is_empty()
                                && tx_event
                                    .send(Ok(ResponseEvent::ReasoningContentDelta {
                                        delta: thinking,
                                        content_index: i64::from(index),
                                    }))
                                    .await
                                    .is_err()
                            {
                                return;
                            }
                            active_blocks.insert(index, ActiveBlock::Thinking);
                        }
                    } // match block
                }
                Ok(AnthropicEvent::ContentBlockDelta { index, delta }) => match delta {
                    orbit_code_anthropic::DeltaType::Text { text } => {
                        if let Some(ActiveBlock::Text { text: accumulator }) =
                            active_blocks.get_mut(&index)
                        {
                            accumulator.push_str(&text);
                        }
                        if tx_event
                            .send(Ok(ResponseEvent::OutputTextDelta(text)))
                            .await
                            .is_err()
                        {
                            return;
                        }
                    }
                    orbit_code_anthropic::DeltaType::InputJson { partial_json } => {
                        if let Some(ActiveBlock::ToolUse { arguments, .. }) =
                            active_blocks.get_mut(&index)
                        {
                            arguments.push_str(&partial_json);
                        }
                    }
                    orbit_code_anthropic::DeltaType::Thinking { thinking } => {
                        if tx_event
                            .send(Ok(ResponseEvent::ReasoningContentDelta {
                                delta: thinking,
                                content_index: i64::from(index),
                            }))
                            .await
                            .is_err()
                        {
                            return;
                        }
                    }
                    orbit_code_anthropic::DeltaType::Signature { .. } => {
                        // Thinking block signature — not needed for event mapping.
                    }
                },
                Ok(AnthropicEvent::ContentBlockStop { index }) => {
                    let Some(active_block) = active_blocks.remove(&index) else {
                        continue;
                    };
                    match active_block {
                        ActiveBlock::Text { text } if !text.is_empty() => {
                            let item = ResponseItem::Message {
                                id: None,
                                role: "assistant".to_string(),
                                content: vec![ContentItem::OutputText { text }],
                                end_turn: None,
                                phase: None,
                            };
                            if tx_event
                                .send(Ok(ResponseEvent::OutputItemDone(item)))
                                .await
                                .is_err()
                            {
                                return;
                            }
                        }
                        ActiveBlock::ToolUse {
                            id,
                            name,
                            arguments,
                        } => {
                            let item = ResponseItem::FunctionCall {
                                id: None,
                                name: if is_oauth {
                                    strip_oauth_tool_prefix(&name)
                                } else {
                                    name
                                },
                                namespace: None,
                                arguments: if arguments.is_empty() {
                                    "{}".to_string()
                                } else {
                                    arguments
                                },
                                call_id: id,
                            };
                            if tx_event
                                .send(Ok(ResponseEvent::OutputItemDone(item)))
                                .await
                                .is_err()
                            {
                                return;
                            }
                        }
                        ActiveBlock::Thinking | ActiveBlock::Text { .. } => {}
                    }
                }
                Ok(AnthropicEvent::MessageDelta {
                    stop_reason: _,
                    usage,
                }) => {
                    if let Some(usage) = usage {
                        input_tokens = input_tokens.max(usage.input_tokens);
                        output_tokens = output_tokens.max(usage.output_tokens);
                    }
                }
                Ok(AnthropicEvent::MessageStop) => {
                    let token_usage = Some(build_token_usage(input_tokens, output_tokens));
                    let event = ResponseEvent::Completed {
                        response_id,
                        token_usage,
                    };
                    let _ = tx_event.send(Ok(event)).await;
                    return;
                }
                Ok(AnthropicEvent::Ping) => {}
                Ok(AnthropicEvent::Error {
                    error_type,
                    message,
                }) => {
                    tracing::debug!(
                        error_type = %error_type,
                        message = %message,
                        "Anthropic SSE error event"
                    );
                    let stream_error = if error_type == "invalid_request_error" {
                        AnthropicError::Api(Box::new(orbit_code_anthropic::AnthropicApiError {
                            status: 400,
                            error_type,
                            message,
                        }))
                    } else {
                        AnthropicError::Api(Box::new(orbit_code_anthropic::AnthropicApiError {
                            status: 500,
                            error_type,
                            message,
                        }))
                    };
                    let _ = tx_event.send(Err(map_anthropic_error(stream_error))).await;
                    return;
                }
                Err(error) => {
                    let _ = tx_event.send(Err(map_anthropic_error(error))).await;
                    return;
                }
            }
        }
    });

    ResponseStream { rx_event }
}

/// Models that use adaptive thinking (type: "adaptive") instead of budgeted thinking.
fn uses_adaptive_thinking(slug: &str) -> bool {
    matches!(slug, "claude-opus-4-6" | "claude-sonnet-4-6")
}

fn requires_1m_context(slug: &str) -> bool {
    // Only opus-4-6 gets the 1M context beta via OAuth.
    // Sonnet-4-6 is rate-limited on long context for Pro/Max subscriptions.
    matches!(slug, "claude-opus-4-6")
}

/// Models that support the `output_config.effort` parameter.
/// Per Anthropic docs: Opus 4.6, Sonnet 4.6, and Opus 4.5 only.
fn supports_effort_parameter(slug: &str) -> bool {
    matches!(
        slug,
        "claude-opus-4-6" | "claude-sonnet-4-6" | "claude-opus-4-5-20251101"
    )
}

fn budgeted_thinking_config(
    max_tokens: u64,
    effort: ReasoningEffortConfig,
) -> Option<ThinkingConfig> {
    let budget_tokens = match effort {
        ReasoningEffortConfig::None => return None,
        ReasoningEffortConfig::Minimal | ReasoningEffortConfig::Low => 2_048,
        ReasoningEffortConfig::Medium => 8_192,
        ReasoningEffortConfig::High => max_tokens.saturating_div(2).saturating_sub(1).min(16_000),
        ReasoningEffortConfig::XHigh => max_tokens.saturating_sub(1).min(31_999),
    };
    Some(ThinkingConfig::Enabled { budget_tokens })
}

fn translate_input_to_messages(
    input: &[ResponseItem],
    system_blocks: &mut Vec<SystemBlock>,
) -> Result<Vec<Message>> {
    let mut messages = Vec::new();

    for item in input {
        match item {
            ResponseItem::Message { role, content, .. } if role == "system" => {
                for block in content_items_to_text_blocks(content) {
                    if let ContentBlock::Text { text, .. } = block {
                        system_blocks.push(SystemBlock {
                            r#type: "text".to_string(),
                            text,
                            cache_control: None,
                        });
                    }
                }
            }
            ResponseItem::Message { role, content, .. } if role == "assistant" => {
                for block in content_items_to_blocks(content)? {
                    append_block(&mut messages, Role::Assistant, block);
                }
            }
            ResponseItem::Message { content, .. } => {
                for block in content_items_to_blocks(content)? {
                    append_block(&mut messages, Role::User, block);
                }
            }
            ResponseItem::FunctionCall {
                name,
                arguments,
                call_id,
                ..
            } => append_block(
                &mut messages,
                Role::Assistant,
                ContentBlock::ToolUse {
                    id: sanitize_tool_call_id(call_id),
                    name: name.clone(),
                    input: serde_json::from_str(arguments)
                        .unwrap_or(Value::String(arguments.clone())),
                },
            ),
            ResponseItem::FunctionCallOutput { call_id, output } => append_block(
                &mut messages,
                Role::User,
                ContentBlock::ToolResult {
                    tool_use_id: sanitize_tool_call_id(call_id),
                    content: output.body.to_text().unwrap_or_else(|| output.to_string()),
                    is_error: output.success.map(|success| !success),
                },
            ),
            ResponseItem::Reasoning { .. }
            | ResponseItem::LocalShellCall { .. }
            | ResponseItem::ToolSearchCall { .. }
            | ResponseItem::CustomToolCall { .. }
            | ResponseItem::CustomToolCallOutput { .. }
            | ResponseItem::ToolSearchOutput { .. }
            | ResponseItem::WebSearchCall { .. }
            | ResponseItem::ImageGenerationCall { .. }
            | ResponseItem::GhostSnapshot { .. }
            | ResponseItem::Compaction { .. }
            | ResponseItem::Other => {}
        }
    }

    Ok(messages)
}

fn content_items_to_blocks(items: &[ContentItem]) -> Result<Vec<ContentBlock>> {
    let mut blocks = Vec::new();
    for item in items {
        match item {
            ContentItem::InputText { text } | ContentItem::OutputText { text } => {
                blocks.push(ContentBlock::Text {
                    text: text.clone(),
                    cache_control: None,
                });
            }
            ContentItem::InputImage { image_url } => {
                blocks.push(ContentBlock::Image {
                    source: parse_image_source(image_url)?,
                });
            }
        }
    }
    Ok(blocks)
}

fn content_items_to_text_blocks(items: &[ContentItem]) -> Vec<ContentBlock> {
    items
        .iter()
        .filter_map(|item| match item {
            ContentItem::InputText { text } | ContentItem::OutputText { text } => {
                Some(ContentBlock::Text {
                    text: text.clone(),
                    cache_control: None,
                })
            }
            ContentItem::InputImage { .. } => None,
        })
        .collect()
}

fn append_block(messages: &mut Vec<Message>, role: Role, block: ContentBlock) {
    if let Some(last_message) = messages.last_mut()
        && last_message.role == role
        && let Content::Blocks(blocks) = &mut last_message.content
    {
        blocks.push(block);
        return;
    }

    messages.push(Message {
        role,
        content: Content::Blocks(vec![block]),
    });
}

fn parse_image_source(image_url: &str) -> Result<ImageSource> {
    let Some(payload) = image_url.strip_prefix("data:") else {
        return Err(CodexErr::InvalidRequest(
            "Anthropic stage 3a requires image inputs to use data URLs.".to_string(),
        ));
    };
    let Some((media_type, data)) = payload.split_once(";base64,") else {
        return Err(CodexErr::InvalidRequest(
            "Anthropic stage 3a requires base64 data URLs for images.".to_string(),
        ));
    };
    Ok(ImageSource {
        r#type: "base64".to_string(),
        media_type: media_type.to_string(),
        data: data.to_string(),
    })
}

fn parse_beta_header(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn build_token_usage(input_tokens: u64, output_tokens: u64) -> TokenUsage {
    let input_tokens = i64::try_from(input_tokens).unwrap_or(i64::MAX);
    let output_tokens = i64::try_from(output_tokens).unwrap_or(i64::MAX);
    TokenUsage {
        input_tokens,
        cached_input_tokens: 0,
        output_tokens,
        reasoning_output_tokens: 0,
        total_tokens: input_tokens.saturating_add(output_tokens),
    }
}

enum ActiveBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        arguments: String,
    },
    Thinking,
}

#[cfg(test)]
mod tests {
    use super::*;
    use orbit_code_anthropic::DeltaType;
    use orbit_code_protocol::models::FunctionCallOutputPayload;
    use pretty_assertions::assert_eq;

    #[test]
    fn converts_local_shell_and_skips_native_web_search() {
        let tools = tools_to_anthropic_format(&[
            ToolSpec::LocalShell {},
            ToolSpec::WebSearch {
                external_web_access: Some(false),
                filters: None,
                user_location: None,
                search_context_size: None,
                search_content_types: None,
            },
        ])
        .expect("tools");

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "local_shell");
    }

    #[test]
    fn sanitizes_tool_call_ids() {
        assert_eq!(
            sanitize_tool_call_id("call.with spaces"),
            "call_with_spaces".to_string()
        );
    }

    #[test]
    fn appends_required_context_beta_header() {
        let mut headers = HeaderMap::new();
        merge_anthropic_beta_headers(
            &mut headers,
            "claude-sonnet-4-6",
            &[CONTEXT_1M_BETA_HEADER_VALUE],
        )
        .expect("headers");

        let beta = headers
            .get("anthropic-beta")
            .and_then(|value| value.to_str().ok())
            .expect("beta header");
        assert!(beta.contains(CONTEXT_1M_BETA_HEADER_VALUE));
    }

    #[tokio::test]
    async fn maps_text_tool_and_thinking_events() {
        // Anthropic streams thinking first, then text, then tool_use.
        let events = futures::stream::iter(vec![
            Ok(AnthropicEvent::MessageStart {
                message_id: "msg_1".to_string(),
                model: "claude-sonnet-4-6".to_string(),
                usage: orbit_code_anthropic::Usage {
                    input_tokens: 12,
                    output_tokens: 0,
                    cache_creation_input_tokens: None,
                    cache_read_input_tokens: None,
                },
            }),
            Ok(AnthropicEvent::ContentBlockStart {
                index: 0,
                block: orbit_code_anthropic::ContentBlockType::Thinking {
                    thinking: String::new(),
                },
            }),
            Ok(AnthropicEvent::ContentBlockDelta {
                index: 0,
                delta: DeltaType::Thinking {
                    thinking: "reasoning".to_string(),
                },
            }),
            Ok(AnthropicEvent::ContentBlockStop { index: 0 }),
            Ok(AnthropicEvent::ContentBlockStart {
                index: 1,
                block: orbit_code_anthropic::ContentBlockType::Text {
                    text: String::new(),
                },
            }),
            Ok(AnthropicEvent::ContentBlockDelta {
                index: 1,
                delta: DeltaType::Text {
                    text: "hello".to_string(),
                },
            }),
            Ok(AnthropicEvent::ContentBlockStop { index: 1 }),
            Ok(AnthropicEvent::ContentBlockStart {
                index: 2,
                block: orbit_code_anthropic::ContentBlockType::ToolUse {
                    id: "call_1".to_string(),
                    name: "shell_command".to_string(),
                },
            }),
            Ok(AnthropicEvent::ContentBlockDelta {
                index: 2,
                delta: DeltaType::InputJson {
                    partial_json: "{\"command\":\"pwd\"}".to_string(),
                },
            }),
            Ok(AnthropicEvent::ContentBlockStop { index: 2 }),
            Ok(AnthropicEvent::MessageDelta {
                stop_reason: Some("end_turn".to_string()),
                usage: Some(orbit_code_anthropic::Usage {
                    input_tokens: 12,
                    output_tokens: 9,
                    cache_creation_input_tokens: None,
                    cache_read_input_tokens: None,
                }),
            }),
            Ok(AnthropicEvent::MessageStop),
        ]);

        let mut stream = map_anthropic_to_response_stream(Box::pin(events), false);
        let mut output_events = Vec::new();
        while let Some(event) = stream.rx_event.recv().await {
            output_events.push(event.expect("ok event"));
        }

        // Created → OutputItemAdded → ReasoningContentDelta → OutputTextDelta
        // → OutputItemDone(Message) → OutputItemDone(FunctionCall) → Completed
        assert!(matches!(output_events[0], ResponseEvent::Created));
        assert!(matches!(
            output_events[1],
            ResponseEvent::OutputItemAdded(ResponseItem::Message { .. })
        ));
        assert!(matches!(
            output_events[2],
            ResponseEvent::ReasoningContentDelta { .. }
        ));
        assert!(matches!(
            output_events[3],
            ResponseEvent::OutputTextDelta(ref delta) if delta == "hello"
        ));
        assert!(matches!(
            output_events[4],
            ResponseEvent::OutputItemDone(ResponseItem::Message { .. })
        ));
        assert!(matches!(
            output_events[5],
            ResponseEvent::OutputItemDone(ResponseItem::FunctionCall { .. })
        ));
        assert!(matches!(output_events[6], ResponseEvent::Completed { .. }));
    }

    #[test]
    fn translates_function_call_output_to_tool_result() {
        let input = vec![
            ResponseItem::FunctionCall {
                id: None,
                name: "test_tool".to_string(),
                namespace: None,
                arguments: "{\"foo\":\"bar\"}".to_string(),
                call_id: "call:1".to_string(),
            },
            ResponseItem::FunctionCallOutput {
                call_id: "call:1".to_string(),
                output: FunctionCallOutputPayload::from_text("ok".to_string()),
            },
        ];
        let mut system = Vec::new();
        let messages = translate_input_to_messages(&input, &mut system).expect("messages");

        assert_eq!(messages.len(), 2);
        assert_eq!(system.len(), 0);
    }
}
