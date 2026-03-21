# codex-rs/otel/

OpenTelemetry instrumentation: OTLP log/trace/metric exporters, session-scoped telemetry, W3C trace context propagation, and event routing (log-only vs trace-safe).

## Build & Test
```bash
cargo build -p orbit-code-otel
cargo test -p orbit-code-otel
```

## Architecture

`OtelProvider` is the top-level entry point -- it builds logger, tracer, and metrics providers from `OtelSettings` and produces tracing-subscriber layers for integration. `SessionTelemetry` provides per-session recording methods for API requests, SSE events, WebSocket events, tool results, user prompts, and auth recovery. `MetricsClient` handles counter/histogram/duration recording.

Events are routed via tracing targets: `orbit_code_otel.log_only` for sensitive data (log export only) and `orbit_code_otel.trace_safe` for data safe to include in distributed traces. W3C trace context (`traceparent`/`tracestate`) is extracted from and injected into HTTP headers and the `TRACEPARENT` environment variable.

## Key Considerations

- The `disable-default-metrics-exporter` feature flag prevents network exports in tests -- always use this feature in `dev-dependencies` to avoid test flakiness.
- OTLP transport supports both gRPC (tonic) and HTTP (proto/JSON) protocols, configured via `OtelExporter` enum. Default metrics exporter targets Statsig.
- `SessionTelemetry` methods emit events on specific tracing targets -- changing the target prefix breaks the log-only vs trace-safe routing.
- The crate depends on `orbit-code-api` for `ResponseEvent` and `ApiError` types, and on `orbit-code-protocol` for thread/session types -- circular dependency would be a problem here.
- TLS configuration for OTLP endpoints uses `orbit-code-utils-absolute-path` for certificate path resolution.
