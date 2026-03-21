# codex-rs/codex-client/

Low-level HTTP transport layer: reqwest-based client, SSE streaming, retry with exponential backoff, custom CA certificate support, zstd compression, and OpenTelemetry request telemetry.

## Build & Test
```bash
cargo build -p orbit-code-client
cargo test -p orbit-code-client
```

## Architecture

The crate defines `HttpTransport`, an async trait for HTTP request execution, with `ReqwestTransport` as the production implementation. This trait boundary enables mock HTTP in tests across the workspace. `CodexHttpClient` wraps a transport with pre-configured defaults (headers, timeouts) and provides `CodexRequestBuilder` for fluent request construction.

Custom CA certificate loading (`custom_ca.rs`) reads from `NODE_EXTRA_CA_CERTS` and merges with system certs to build a rustls `ClientConfig`. Retry logic (`retry.rs`) implements exponential backoff with jitter. SSE stream conversion (`sse.rs`) wraps HTTP response byte streams into parsed events. Request telemetry (`telemetry.rs`) captures metadata for OpenTelemetry spans.

## Key Considerations

- `HttpTransport` trait is the mock boundary for all HTTP in the workspace -- `orbit-code-api` and `orbit-code-core` depend on it for test isolation.
- Custom CA loading supports `NODE_EXTRA_CA_CERTS` (the Node.js convention) for corporate proxy compatibility. The `custom_ca_probe` binary in `src/bin/` is a diagnostic tool for verifying CA setup.
- Retry policy uses `2^(attempt-1)` backoff with +/-10% jitter. Only retries transient errors (429, 5xx, connection failures).
- Request compression uses zstd when `RequestCompression::Zstd` is specified.
- `StreamError` and `TransportError` are distinct error types -- `StreamError` is for SSE-specific failures, `TransportError` for HTTP-level issues.
