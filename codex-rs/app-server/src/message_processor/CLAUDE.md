# app-server/src/message_processor

## Purpose

Contains submodules for the `MessageProcessor`, which is the top-level request router in the app-server. Currently holds test modules that exercise tracing and instrumentation behavior.

## Key Files

| File | Role |
|------|------|
| `tracing_tests.rs` | Unit tests verifying that the `MessageProcessor` correctly creates and propagates tracing spans for incoming requests. Tests cover request span creation with W3C trace context, typed request span creation, thread/start and turn/start flows against a mock model server. |

## What It Plugs Into

- This module is included in the parent `message_processor.rs` via `#[cfg(test)] mod tracing_tests;`.
- Tests use `app_test_support` for mock model servers and config helpers.

## Imports From

- `super::*` -- Parent `MessageProcessor`, `ConnectionSessionState`, `MessageProcessorArgs`.
- `crate::outgoing_message` -- `ConnectionId`, `OutgoingMessageSender`.
- `crate::transport::AppServerTransport` -- Transport enum for test request processing.
- `codex-app-server-protocol` -- Request/response types for initialize, thread/start, turn/start.
- `app_test_support` -- Test fixture helpers (mock model server, config writing).
- `opentelemetry`, `tracing-opentelemetry` -- OTel integration for span verification.

## Exports To

- No exports; this is a test-only submodule.
