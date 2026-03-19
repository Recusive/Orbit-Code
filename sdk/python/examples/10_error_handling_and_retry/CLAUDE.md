# sdk/python/examples/10_error_handling_and_retry/

Demonstrates error handling and the retry-on-overload pattern.

## Purpose

Shows how to use `retry_on_overload` for transient server errors, and how to handle typed exceptions from the SDK error hierarchy (`ServerBusyError`, `JsonRpcError`, etc.).

## Key Files

- `sync.py` -- Synchronous version
- `async.py` -- Async version

## Imports From

- `_bootstrap` for setup helpers
- `codex_app_server.Codex`, `codex_app_server.retry_on_overload`, error classes
