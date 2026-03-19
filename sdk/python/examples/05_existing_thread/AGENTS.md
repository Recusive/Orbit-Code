# sdk/python/examples/05_existing_thread/

This file applies to `sdk/python/examples/05_existing_thread/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- This subtree belongs to the `codex-app-server-sdk` Python package. Keep import surfaces and packaging metadata consistent when you move or rename modules.
- Examples should stay minimal, runnable, and aligned with the public SDK surface. Keep sync and async examples consistent when both exist.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/sdk/python && pytest`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Demonstrates resuming an existing thread by ID.

### Purpose

Shows how to create a thread, note its ID, then resume it with `codex.thread_resume(thread_id)` to continue the conversation.

### Key Files

- `sync.py` -- Synchronous version
- `async.py` -- Async version

### Imports From

- `_bootstrap` for setup helpers
- `codex_app_server.Codex` / `codex_app_server.AsyncCodex`
