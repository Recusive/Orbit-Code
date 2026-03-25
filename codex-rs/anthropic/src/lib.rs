//! Anthropic Messages API client with typed SSE parsing.

mod client;
mod error;
mod models;
mod stream;
mod token_refresh;
mod types;

pub use client::AnthropicAuth;
pub use client::AnthropicClient;
pub use client::AnthropicStream;
pub use error::AnthropicApiError;
pub use error::AnthropicError;
pub use error::Result;
pub use models::AnthropicCapabilities;
pub use models::AnthropicModelInfo;
pub use models::AnthropicModelsClient;
pub use models::AnthropicModelsError;
pub use models::AnthropicModelsResponse;
pub use stream::AnthropicEvent;
pub use stream::ContentBlockType;
pub use stream::DeltaType;
pub use stream::Usage;
pub use stream::parse_sse_event;
pub use token_refresh::RefreshedTokens;
pub use token_refresh::refresh_anthropic_token;
pub use types::CacheControl;
pub use types::Content;
pub use types::ContentBlock;
pub use types::ImageSource;
pub use types::Message;
pub use types::MessagesRequest;
pub use types::OutputConfig;
pub use types::Role;
pub use types::SystemBlock;
pub use types::ThinkingConfig;
pub use types::ThinkingDisplay;
pub use types::Tool;
pub use types::ToolChoice;

pub const ANTHROPIC_VERSION_HEADER_VALUE: &str = "2023-06-01";
pub const ANTHROPIC_BETA_HEADER_VALUE: &str =
    "claude-code-20250219,interleaved-thinking-2025-05-14,fine-grained-tool-streaming-2025-05-14";
pub const CONTEXT_1M_BETA_HEADER_VALUE: &str = "context-1m-2025-08-07";
