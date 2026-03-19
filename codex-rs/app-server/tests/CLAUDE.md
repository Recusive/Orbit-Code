# app-server/tests

## Purpose

Integration test suite for the `codex-app-server` crate. Tests exercise the full server stack by spawning a real app-server process (or using in-process transport) and communicating over JSON-RPC via stdio or WebSocket.

## Structure

- **`all.rs`** -- Single integration test binary entry point that aggregates all test modules. Declares `mod suite;` to pull in the test suite directory.
- **`common/`** -- Shared test utilities library (`app_test_support` crate) providing mock servers, auth fixtures, config helpers, and response builders.
- **`suite/`** -- Test modules organized by feature area, with a `v2/` subdirectory for v2 protocol tests.

## What It Plugs Into

- Tests depend on `app_test_support` (from `common/`) for shared infrastructure.
- They spawn or embed the `codex-app-server` binary and communicate via JSON-RPC.
- The test suite uses `wiremock` for HTTP mocking, `serial_test` for tests that need exclusive resource access, and `codex-utils-cargo-bin` for locating test binaries.

## Imports From

- `codex-app-server-protocol` -- Message types for constructing requests and parsing responses.
- `app_test_support` (`common/`) -- Mock model server, auth fixtures, config helpers.
- `core_test_support` -- Shell formatting and temp path helpers.
- `codex-utils-cargo-bin` -- Binary path resolution for test processes.

## Exports To

- No exports; this is a test-only directory.
