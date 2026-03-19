# sdk/python/examples/02_turn_run/

Demonstrates inspecting full turn output fields from `thread.run()`.

## Purpose

Shows how to examine the `RunResult` object returned by `thread.run()`, including `final_response`, `items`, and `usage`.

## Key Files

- `sync.py` -- Synchronous version
- `async.py` -- Async version

## Imports From

- `_bootstrap` for setup helpers
- `codex_app_server.Codex` / `codex_app_server.AsyncCodex`
