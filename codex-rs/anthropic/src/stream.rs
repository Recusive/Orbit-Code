//! Typed SSE event parsing for Anthropic Messages streams.

use crate::error::AnthropicError;
use serde::Deserialize;

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Usage {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: Option<u64>,
    #[serde(default)]
    pub cache_read_input_tokens: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnthropicEvent {
    MessageStart {
        message_id: String,
        model: String,
        usage: Usage,
    },
    ContentBlockStart {
        index: u32,
        block: ContentBlockType,
    },
    ContentBlockDelta {
        index: u32,
        delta: DeltaType,
    },
    ContentBlockStop {
        index: u32,
    },
    MessageDelta {
        stop_reason: Option<String>,
        usage: Option<Usage>,
    },
    MessageStop,
    Ping,
    Error {
        error_type: String,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContentBlockType {
    Text { text: String },
    ToolUse { id: String, name: String },
    Thinking { thinking: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeltaType {
    Text { text: String },
    InputJson { partial_json: String },
    Thinking { thinking: String },
    /// Signature for thinking block verification. Ignored by the bridge.
    Signature { signature: String },
}

pub fn parse_sse_event(
    event_name: &str,
    data: &str,
) -> Result<Option<AnthropicEvent>, AnthropicError> {
    match event_name {
        "message_start" => {
            let payload: MessageStartPayload = serde_json::from_str(data)?;
            Ok(Some(AnthropicEvent::MessageStart {
                message_id: payload.message.id,
                model: payload.message.model,
                usage: payload.message.usage,
            }))
        }
        "content_block_start" => {
            let payload: ContentBlockStartPayload = serde_json::from_str(data)?;
            Ok(Some(AnthropicEvent::ContentBlockStart {
                index: payload.index,
                block: payload.content_block.into_content_block_type()?,
            }))
        }
        "content_block_delta" => {
            let payload: ContentBlockDeltaPayload = serde_json::from_str(data)?;
            Ok(Some(AnthropicEvent::ContentBlockDelta {
                index: payload.index,
                delta: payload.delta.into_delta_type()?,
            }))
        }
        "content_block_stop" => {
            let payload: ContentBlockStopPayload = serde_json::from_str(data)?;
            Ok(Some(AnthropicEvent::ContentBlockStop {
                index: payload.index,
            }))
        }
        "message_delta" => {
            let payload: MessageDeltaPayload = serde_json::from_str(data)?;
            Ok(Some(AnthropicEvent::MessageDelta {
                stop_reason: payload.delta.stop_reason,
                usage: payload.usage,
            }))
        }
        "message_stop" => Ok(Some(AnthropicEvent::MessageStop)),
        "ping" => Ok(Some(AnthropicEvent::Ping)),
        "error" => {
            let payload: ErrorPayload = serde_json::from_str(data)?;
            Ok(Some(AnthropicEvent::Error {
                error_type: payload.error.r#type,
                message: payload.error.message,
            }))
        }
        _ => Ok(None),
    }
}

#[derive(Debug, Deserialize)]
struct MessageStartPayload {
    message: MessageStartBody,
}

#[derive(Debug, Deserialize)]
struct MessageStartBody {
    id: String,
    model: String,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct ContentBlockStartPayload {
    index: u32,
    content_block: RawContentBlockType,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum RawContentBlockType {
    Text { text: String },
    ToolUse { id: String, name: String },
    Thinking { thinking: String },
}

impl RawContentBlockType {
    fn into_content_block_type(self) -> Result<ContentBlockType, AnthropicError> {
        Ok(match self {
            Self::Text { text } => ContentBlockType::Text { text },
            Self::ToolUse { id, name } => ContentBlockType::ToolUse { id, name },
            Self::Thinking { thinking } => ContentBlockType::Thinking { thinking },
        })
    }
}

#[derive(Debug, Deserialize)]
struct ContentBlockDeltaPayload {
    index: u32,
    delta: RawDeltaType,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum RawDeltaType {
    #[serde(rename = "text_delta")]
    Text { text: String },
    #[serde(rename = "input_json_delta")]
    InputJson { partial_json: String },
    #[serde(rename = "thinking_delta")]
    Thinking { thinking: String },
    #[serde(rename = "signature_delta")]
    Signature { signature: String },
}

impl RawDeltaType {
    fn into_delta_type(self) -> Result<DeltaType, AnthropicError> {
        Ok(match self {
            Self::Text { text } => DeltaType::Text { text },
            Self::InputJson { partial_json } => DeltaType::InputJson { partial_json },
            Self::Thinking { thinking } => DeltaType::Thinking { thinking },
            Self::Signature { signature } => DeltaType::Signature { signature },
        })
    }
}

#[derive(Debug, Deserialize)]
struct ContentBlockStopPayload {
    index: u32,
}

#[derive(Debug, Deserialize)]
struct MessageDeltaPayload {
    delta: MessageDeltaBody,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct MessageDeltaBody {
    #[serde(default)]
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ErrorPayload {
    error: ErrorBody,
}

#[derive(Debug, Deserialize)]
struct ErrorBody {
    r#type: String,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parses_message_start_event() {
        let event = parse_sse_event(
            "message_start",
            r#"{"message":{"id":"msg_1","model":"claude-sonnet-4-6","usage":{"input_tokens":12,"output_tokens":0}}}"#,
        )
        .expect("parse")
        .expect("event");

        assert_eq!(
            event,
            AnthropicEvent::MessageStart {
                message_id: "msg_1".to_string(),
                model: "claude-sonnet-4-6".to_string(),
                usage: Usage {
                    input_tokens: 12,
                    output_tokens: 0,
                    cache_creation_input_tokens: None,
                    cache_read_input_tokens: None,
                },
            }
        );
    }

    #[test]
    fn parses_tool_use_delta_event() {
        let event = parse_sse_event(
            "content_block_delta",
            r#"{"index":1,"delta":{"type":"input_json_delta","partial_json":"{\"foo\":\"bar\"}"}}"#,
        )
        .expect("parse")
        .expect("event");

        assert_eq!(
            event,
            AnthropicEvent::ContentBlockDelta {
                index: 1,
                delta: DeltaType::InputJson {
                    partial_json: "{\"foo\":\"bar\"}".to_string(),
                },
            }
        );
    }

    #[test]
    fn parses_thinking_start_event() {
        let event = parse_sse_event(
            "content_block_start",
            r#"{"index":2,"content_block":{"type":"thinking","thinking":"step one"}}"#,
        )
        .expect("parse")
        .expect("event");

        assert_eq!(
            event,
            AnthropicEvent::ContentBlockStart {
                index: 2,
                block: ContentBlockType::Thinking {
                    thinking: "step one".to_string(),
                },
            }
        );
    }

    #[test]
    fn parses_error_event() {
        let event = parse_sse_event(
            "error",
            r#"{"error":{"type":"invalid_request_error","message":"bad request"}}"#,
        )
        .expect("parse")
        .expect("event");

        assert_eq!(
            event,
            AnthropicEvent::Error {
                error_type: "invalid_request_error".to_string(),
                message: "bad request".to_string(),
            }
        );
    }

    #[test]
    fn adaptive_thinking_serializes_without_effort() {
        let value = serde_json::to_value(crate::ThinkingConfig::Adaptive {}).expect("serialize");
        assert_eq!(value, serde_json::json!({ "type": "adaptive" }));
    }
}
