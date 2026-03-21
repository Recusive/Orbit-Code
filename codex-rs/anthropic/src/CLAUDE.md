# codex-rs/anthropic/src/

Implementation of the Anthropic Messages API client with typed SSE streaming.

## Module Layout

- **client** (`client.rs`) -- `AnthropicClient` and `AnthropicAuth` (API key vs OAuth); builds and sends streaming requests, returns `AnthropicStream`
- **stream** (`stream.rs`) -- SSE event parsing; converts raw events into typed `AnthropicEvent` variants (message lifecycle, content blocks, deltas, usage)
- **types** (`types.rs`) -- Request/response data types: `MessagesRequest`, `Message`, `Content`, `Tool`, `ToolChoice`, `SystemBlock`, `ThinkingConfig`, etc.
- **error** (`error.rs`) -- `AnthropicError` enum with API, transport, and parse variants; `AnthropicApiError` for structured API error responses
- **token_refresh** (`token_refresh.rs`) -- `refresh_anthropic_token()` for OAuth token refresh via Anthropic's token endpoint
