# codex-rs/otel/src/events/

This file applies to `codex-rs/otel/src/events/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-otel` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-otel`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Session-scoped telemetry event emission for the Codex CLI.

### What this folder does

Provides the `SessionTelemetry` struct and supporting macros that emit structured telemetry events for Codex sessions. Events are emitted to both log-only and trace-safe targets with different detail levels -- log events include full request/response content (for internal observability), while trace events include only safe aggregates (lengths, counts, durations).

### Key files

- `mod.rs` -- declares submodules `session_telemetry` and `shared`; both are `pub(crate)`
- `session_telemetry.rs` -- `SessionTelemetry` struct and all its recording methods:
  - `new()` -- constructs with conversation ID, model, auth mode, originator, etc.
  - `conversation_starts()` -- logs session start with provider, reasoning, approval/sandbox policy, MCP servers
  - `record_api_request()` -- logs HTTP API calls with status, duration, auth details
  - `record_websocket_connect()` / `record_websocket_request()` / `record_websocket_event()` -- WebSocket lifecycle events
  - `log_sse_event()` / `sse_event_completed()` -- SSE streaming events
  - `user_prompt()` -- logs user input (full text to logs, only length/counts to traces)
  - `tool_decision()` / `tool_result_with_tags()` / `log_tool_result_with_tags()` -- tool call telemetry
  - `record_auth_recovery()` -- auth recovery lifecycle events
  - `record_responses()` -- response event span recording
  - `counter()` / `histogram()` / `record_duration()` / `start_timer()` -- metrics forwarding with session metadata tags
  - `runtime_metrics_summary()` / `reset_runtime_metrics()` -- runtime metric snapshots
- `shared.rs` -- `log_event!`, `trace_event!`, and `log_and_trace_event!` macros that emit events to the appropriate tracing targets with shared metadata fields (conversation ID, app version, auth mode, originator, model, terminal type, timestamps)

### Imports from

- `crate::metrics` -- `MetricsClient`, `MetricsConfig`, `MetricsError`, metric name constants, `Timer`, `RuntimeMetricsSummary`, `SessionMetricTagValues`
- `crate::provider` -- `OtelProvider`
- `crate::targets` -- `OTEL_LOG_ONLY_TARGET`, `OTEL_TRACE_SAFE_TARGET`
- `codex_api` -- `ResponseEvent`, `ApiError`
- `codex_protocol` -- `ThreadId`, `ResponseItem`, `ReviewDecision`, `SandboxPolicy`, `SessionSource`, `UserInput`, `ReasoningSummary`, `ReasoningEffort`, `AskForApproval`

### Exports to

- `SessionTelemetry`, `SessionTelemetryMetadata`, `AuthEnvTelemetryMetadata` are re-exported through `lib.rs`
- Macros are `pub(crate)` only, used within the events module
