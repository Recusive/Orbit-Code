# sdk/python/examples/05_existing_thread/

Demonstrates resuming an existing thread by ID.

## Purpose

Shows how to create a thread, note its ID, then resume it with `codex.thread_resume(thread_id)` to continue the conversation.

## Key Files

- `sync.py` -- Synchronous version
- `async.py` -- Async version

## Imports From

- `_bootstrap` for setup helpers
- `codex_app_server.Codex` / `codex_app_server.AsyncCodex`
