# sdk/python/examples/14_turn_controls/

Demonstrates turn steering and interruption.

## Purpose

Shows how to use `TurnHandle.steer()` to redirect an in-progress turn and `TurnHandle.interrupt()` to cancel one. Both are best-effort operations.

## Key Files

- `sync.py` -- Synchronous version demonstrating both steer and interrupt
- `async.py` -- Async version

## Imports From

- `_bootstrap` for setup helpers
- `codex_app_server.Codex`, `codex_app_server.TextInput`
