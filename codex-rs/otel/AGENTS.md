# codex-rs/otel/

This file applies to `codex-rs/otel/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-otel` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-otel`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

OpenTelemetry instrumentation crate (`codex-otel`) providing logging, tracing, and metrics for the Codex CLI.

### What this folder does

This crate wraps the OpenTelemetry SDK to provide a unified observability layer for the Codex CLI. It handles:
- Configuring and building OTLP log, trace, and metric exporters (gRPC and HTTP)
- Session-scoped telemetry (API calls, SSE events, WebSocket events, tool calls)
- Metrics collection with counters, histograms, and duration timers
- W3C trace context propagation (traceparent/tracestate)
- Event routing: sensitive data goes to log-only export; safe data goes to both logs and traces

### What it plugs into

- Consumed by `codex-core` and other higher-level crates for session instrumentation
- Exports to OTLP-compatible collectors (Statsig by default for metrics, configurable endpoints for logs/traces)
- Uses `tracing` and `tracing-subscriber` for structured event emission with target-based filtering

### Imports from

- `codex-protocol` -- `W3cTraceContext`, `ThreadId`, `SessionSource`, `ResponseItem`, `ReviewDecision`, `SandboxPolicy`, etc.
- `codex-api` -- `ResponseEvent`, `ApiError`
- `codex-utils-absolute-path` -- TLS certificate path handling
- `codex-utils-string` -- `sanitize_metric_tag_value`
- `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp` -- core SDK
- `tracing`, `tracing-subscriber`, `tracing-opentelemetry` -- structured logging and bridging

### Exports to

- `SessionTelemetry` -- per-session telemetry recorder with methods for API requests, SSE events, WebSocket events, tool results, user prompts, and auth recovery
- `OtelProvider` -- top-level provider that builds and manages logger, tracer, and metrics providers
- `MetricsClient` / `MetricsConfig` -- metrics client for counters, histograms, duration recording
- `Timer` -- RAII-based duration timer
- `RuntimeMetricsSummary` / `RuntimeMetricTotals` -- snapshot summaries for per-turn runtime metrics
- Trace context utilities: `context_from_w3c_trace_context`, `current_span_trace_id`, `set_parent_from_w3c_trace_context`, `traceparent_context_from_env`

### Key files

- `Cargo.toml` -- crate definition; feature `disable-default-metrics-exporter` disables network exports in tests
- `src/lib.rs` -- public API surface; re-exports from submodules
- `src/config.rs` -- `OtelSettings`, `OtelExporter`, `OtelTlsConfig`; Statsig endpoint defaults
- `src/provider.rs` -- `OtelProvider::from()` builds logger, tracer, and metrics providers from settings
- `src/otlp.rs` -- HTTP/gRPC TLS client builders, header map construction, timeout resolution
- `src/targets.rs` -- target prefixes (`codex_otel.log_only`, `codex_otel.trace_safe`) for routing events
- `src/trace_context.rs` -- W3C trace context extraction, injection, and `TRACEPARENT` env var support
- `src/events/` -- telemetry event emission (session-scoped)
- `src/metrics/` -- metrics client, config, validation, timer, runtime summary
- `tests/` -- integration tests for metrics, OTLP HTTP loopback, event routing, snapshots
