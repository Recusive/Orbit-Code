# codex-rs/codex-api/src/

Implementation of all OpenAI API clients, organized by endpoint, transport, and shared types.

## Module Layout

- **endpoint/** -- Per-endpoint client implementations: `ResponsesClient` (HTTP SSE), `ResponsesWebsocketClient`, `RealtimeWebsocketClient`, `CompactClient`, `MemoriesClient`, `ModelsClient`
- **requests/** -- Request construction: header building, body formatting, API version handling
- **sse/** -- SSE stream processing: `process_sse`, `spawn_response_stream`, fixture helpers for testing
- **provider** (`provider.rs`) -- `Provider` struct for base URL, auth, and API version; Azure URL detection
- **auth** (`auth.rs`) -- `AuthProvider` trait for supplying bearer tokens to requests
- **common** (`common.rs`) -- Shared types: `ResponsesApiRequest`, `ResponseEvent`, `ResponseStream`, `CompactionInput`, memory types
- **error** (`error.rs`) -- `ApiError` enum covering transport, SSE, WebSocket, and API-level errors
- **telemetry** (`telemetry.rs`) -- `SseTelemetry` and `WebsocketTelemetry` for per-request metric capture
