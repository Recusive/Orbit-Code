# codex-rs/codex-api/

API client library for OpenAI endpoints: Responses API (HTTP SSE + WebSocket), Realtime API (WebSocket), models listing, memory summarization, and context compaction.

## Build & Test
```bash
cargo build -p orbit-code-api
cargo test -p orbit-code-api
```

## Architecture

The crate is organized around endpoint-specific clients (`ResponsesClient`, `ResponsesWebsocketClient`, `RealtimeWebsocketClient`, `CompactClient`, `MemoriesClient`, `ModelsClient`), each living in `src/endpoint/`. All HTTP clients build on `orbit-code-client`'s `ReqwestTransport` trait. WebSocket clients use `tokio-tungstenite` directly.

The `Provider` struct encapsulates base URL, auth, and API version configuration. `AuthProvider` is a trait that supplies bearer tokens for requests. Request construction (headers, body formatting) lives in `src/requests/`. SSE stream processing lives in `src/sse/` and provides `process_sse` and `spawn_response_stream` for consuming streamed responses.

## Key Considerations

- `Provider` handles Azure vs OpenAI URL differences via `is_azure_responses_wire_base_url` -- Azure uses a different path structure for the Responses API.
- SSE helpers (`stream_from_fixture`, `process_sse`) are pub and used extensively in test code across the workspace -- changes here ripple into `orbit-code-core` tests.
- WebSocket clients support both Realtime API v1/v2 protocols -- the `RealtimeSessionConfig` controls which protocol variant to use.
- Rate limit parsing (`rate_limits.rs`) extracts headers from API responses for backpressure signaling.
- `SseTelemetry` and `WebsocketTelemetry` capture per-request metrics consumed by `orbit-code-otel`.
