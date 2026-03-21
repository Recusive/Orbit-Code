# codex-rs/anthropic/

Typed Rust client for the Anthropic Messages API with SSE streaming and token refresh support.

## Build & Test
```bash
cargo build -p orbit-code-anthropic
cargo test -p orbit-code-anthropic
```

## Architecture

The crate provides `AnthropicClient` which sends streaming requests to the Anthropic Messages API and returns an `AnthropicStream` of typed `AnthropicEvent` values parsed from SSE. Authentication is handled via `AnthropicAuth` (API key or OAuth token). The `token_refresh` module supports refreshing expired OAuth tokens via Anthropic's token endpoint.

Request/response types (`MessagesRequest`, `Message`, `Content`, `Tool`, etc.) live in `types.rs`. SSE parsing logic that converts raw server-sent events into `AnthropicEvent` variants lives in `stream.rs`. Error handling uses `AnthropicError` with variants for API errors, transport failures, and parse errors.

## Key Considerations

- Built on top of `orbit-code-client` for HTTP transport (not raw reqwest) -- uses `ReqwestTransport` trait for testability.
- SSE parsing (`parse_sse_event`) must handle Anthropic-specific event types: `message_start`, `content_block_start`, `content_block_delta`, `content_block_stop`, `message_delta`, `message_stop`, `error`, and `ping`.
- The `ANTHROPIC_BETA_HEADER_VALUE` constant in `lib.rs` controls which beta features are enabled -- update it when adopting new Anthropic API betas.
- Token refresh (`refresh_anthropic_token`) is a standalone async function, not tied to the client -- callers in `orbit-code-core` use it to refresh tokens before they expire.
