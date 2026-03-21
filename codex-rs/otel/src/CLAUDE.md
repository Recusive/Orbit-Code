# codex-rs/otel/src/

OpenTelemetry provider, session telemetry, metrics, and trace context implementation.

## Module Layout

- **provider** (`provider.rs`) -- `OtelProvider`: builds logger, tracer, and metrics providers from `OtelSettings`; produces `logger_layer()` and `tracing_layer()` for subscriber composition
- **config** (`config.rs`) -- `OtelSettings`, `OtelExporter` enum (None/Statsig/OtlpGrpc/OtlpHttp), `OtelTlsConfig`, Statsig endpoint defaults
- **events/** -- Session-scoped telemetry event recording (`SessionTelemetry`) with methods for API calls, SSE events, tool results, user prompts
- **metrics/** -- `MetricsClient`, `MetricsConfig`, validation, `Timer` (RAII duration), `RuntimeMetricsSummary`, metric name constants
- **targets** (`targets.rs`) -- Tracing target prefixes for log-only vs trace-safe event routing
- **trace_context** (`trace_context.rs`) -- W3C traceparent/tracestate extraction, injection, and `TRACEPARENT` env var support
- **otlp** (`otlp.rs`) -- Low-level OTLP transport: gRPC TLS config, HTTP client builders, header map construction, timeout resolution
