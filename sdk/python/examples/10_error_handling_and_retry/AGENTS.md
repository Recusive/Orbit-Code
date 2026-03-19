# sdk/python/examples/10_error_handling_and_retry/

This file applies to `sdk/python/examples/10_error_handling_and_retry/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- This subtree belongs to the `codex-app-server-sdk` Python package. Keep import surfaces and packaging metadata consistent when you move or rename modules.
- Examples should stay minimal, runnable, and aligned with the public SDK surface. Keep sync and async examples consistent when both exist.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/sdk/python && pytest`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Demonstrates error handling and the retry-on-overload pattern.

### Purpose

Shows how to use `retry_on_overload` for transient server errors, and how to handle typed exceptions from the SDK error hierarchy (`ServerBusyError`, `JsonRpcError`, etc.).

### Key Files

- `sync.py` -- Synchronous version
- `async.py` -- Async version

### Imports From

- `_bootstrap` for setup helpers
- `codex_app_server.Codex`, `codex_app_server.retry_on_overload`, error classes
