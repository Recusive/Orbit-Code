# codex-rs/codex-client/src/

HTTP transport, custom CA, retry, SSE processing, and telemetry implementations.

## Module Layout

- **transport** (`transport.rs`) -- `HttpTransport` async trait and `ReqwestTransport` implementation; `ByteStream` and `StreamResponse` types
- **default_client** (`default_client.rs`) -- `CodexHttpClient` pre-configured client with `CodexRequestBuilder` fluent API
- **custom_ca** (`custom_ca.rs`) -- Loads CAs from `NODE_EXTRA_CA_CERTS`, merges with system certs, builds rustls `ClientConfig`
- **request** (`request.rs`) -- `Request`/`Response` types with optional zstd compression support
- **retry** (`retry.rs`) -- `RetryPolicy`, `RetryOn` enum, `run_with_retry` executor with exponential `backoff`
- **sse** (`sse.rs`) -- Converts HTTP response streams into parsed SSE events
- **error** (`error.rs`) -- `TransportError` (HTTP-level) and `StreamError` (SSE-specific)
- **telemetry** (`telemetry.rs`) -- `RequestTelemetry` for OpenTelemetry span instrumentation
