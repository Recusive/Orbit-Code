# codex-rs/otel/tests/suite/

Integration test suites for the `codex-otel` crate.

## What this folder does

Contains individual test modules that cover different aspects of the metrics and telemetry system. Each module focuses on a specific concern.

## Key files

- `mod.rs` -- declares all test submodules
- `validation.rs` -- tests that invalid metric names, tag keys, tag values, and negative counter increments are rejected by the `MetricsClient`
- `send.rs` -- tests that counters and histograms are emitted with correct tag merging (defaults + per-call overrides), and that shutdown flushes the in-memory exporter
- `timing.rs` -- tests for `record_duration()` histogram output and the RAII `Timer` (drop-based recording)
- `snapshot.rs` -- tests for `MetricsClient::snapshot()` (runtime reader) and `SessionTelemetry::snapshot_metrics()` collecting data without shutdown
- `otel_export_routing_policy.rs` -- verifies the dual-sink event routing: sensitive fields (prompt text, tool arguments/output, email) go only to log export; safe aggregates go to trace export. Tests cover `user_prompt`, `tool_result`, `auth_recovery`, `api_request`, `websocket_connect`, and `websocket_request` events
- `otlp_http_loopback.rs` -- end-to-end tests that spin up local TCP servers and verify OTLP HTTP JSON export for both metrics and traces (including multi-thread and current-thread tokio runtimes)
- `runtime_summary.rs` -- tests `RuntimeMetricsSummary` aggregation across tool calls, API calls, SSE events, WebSocket events, and Responses API timing metrics
- `manager_metrics.rs` -- tests that `SessionTelemetry` attaches session metadata tags to forwarded metrics, supports disabling metadata tags, and attaches optional `service_name` tags

## Imports from

- `crate::harness` -- shared test helpers
- `codex_otel` -- `SessionTelemetry`, `OtelProvider`, `MetricsClient`, `MetricsConfig`, `RuntimeMetricsSummary`, etc.
- `codex_protocol` -- `ThreadId`, `SessionSource`, `SandboxPolicy`, `AskForApproval`, `UserInput`, etc.
- `opentelemetry_sdk` -- in-memory exporters and metric data types

## Exports to

Test-only; no production exports.
