# codex-rs/core/tests/

Integration test suite and test infrastructure for `codex-core`.

## What this folder does

Contains the integration test binary, shared test utilities, test fixtures, and the comprehensive test suite. Tests are compiled into a single binary via `all.rs` for faster linking and execution.

## Directory structure

| Directory/File | Purpose |
|----------------|---------|
| `all.rs` | Single integration test binary entry point, aggregates all test modules via `mod suite;` |
| `common/` | Shared test utilities library (`core_test_support` crate) |
| `fixtures/` | Test fixture data files |
| `suite/` | All integration test modules |
| `cli_responses_fixture.sse` | SSE response fixture for CLI tests |
| `responses_headers.rs` | HTTP response header helpers |

## Test architecture

- Tests use `wiremock` for HTTP server mocking (simulating the OpenAI API)
- `core_test_support` provides helpers for creating test sessions, mock SSE streams, and config builders
- Most tests follow the pattern: create a mock server, configure responses, create a `CodexThread`, send messages, assert events
- Insta snapshots are used for complex output validation

## How to run

```bash
# Run all codex-core integration tests
cargo test -p codex-core

# Run with nextest (preferred)
cargo nextest run -p codex-core

# Run a specific test
cargo test -p codex-core -- suite::tools
```

## Imports from

- `codex_core` -- The library being tested
- `core_test_support` -- Test utilities (defined in `common/`)
- `wiremock` -- HTTP mocking
- `insta` -- Snapshot testing
- `tempfile` -- Temporary directories

## Exports to

This is a test-only directory; it does not export to production code.
