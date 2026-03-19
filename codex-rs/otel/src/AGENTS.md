# codex-rs/otel/src/

This file applies to `codex-rs/otel/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-otel` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-otel`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source code for the `codex-otel` crate -- OpenTelemetry instrumentation for the Codex CLI.

### What this folder does

Contains the library source for configuring OTLP exporters, emitting structured telemetry events, managing metrics, and propagating W3C trace context.

### Module structure

- `lib.rs` -- crate root; declares public modules (`config`, `metrics`, `provider`, `trace_context`) and private modules (`events`, `otlp`, `targets`); re-exports key types (`SessionTelemetry`, `OtelProvider`, `Timer`, `RuntimeMetricsSummary`, etc.)
- `config.rs` -- `OtelSettings` (environment, service, exporter config), `OtelExporter` enum (None, Statsig, OtlpGrpc, OtlpHttp), `OtelTlsConfig`, `resolve_exporter()` for Statsig defaults
- `provider.rs` -- `OtelProvider` struct holding optional `SdkLoggerProvider`, `SdkTracerProvider`, `Tracer`, and `MetricsClient`; `from()` factory builds all three from `OtelSettings`; provides `logger_layer()` and `tracing_layer()` for subscriber composition
- `otlp.rs` -- low-level OTLP transport: `build_header_map()`, `build_grpc_tls_config()`, `build_http_client()` (blocking), `build_async_http_client()`, `resolve_otlp_timeout()`
- `targets.rs` -- constants `OTEL_LOG_ONLY_TARGET` and `OTEL_TRACE_SAFE_TARGET`; filter functions `is_log_export_target()` and `is_trace_safe_target()` that control which events go to log export vs trace export
- `trace_context.rs` -- W3C traceparent/tracestate extraction and injection; `traceparent_context_from_env()` reads `TRACEPARENT`/`TRACESTATE` env vars
- `events/` -- telemetry event recording macros and session telemetry
- `metrics/` -- metrics client, configuration, validation, timer, metric names, and runtime summaries

### Imports from

- `codex-protocol`, `codex-api`, `codex-utils-absolute-path`, `codex-utils-string`
- `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp`, `opentelemetry-appender-tracing`, `opentelemetry-semantic-conventions`
- `tracing`, `tracing-subscriber`, `tracing-opentelemetry`
- `reqwest`, `http`, `gethostname`, `os_info`, `chrono`, `serde`, `serde_json`, `tokio`, `tokio-tungstenite`

### Exports to

All public types are re-exported through `lib.rs` and consumed by `codex-core` and other crates in the workspace.
