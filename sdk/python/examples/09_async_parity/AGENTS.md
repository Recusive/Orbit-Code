# sdk/python/examples/09_async_parity/

This file applies to `sdk/python/examples/09_async_parity/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- This subtree belongs to the `codex-app-server-sdk` Python package. Keep import surfaces and packaging metadata consistent when you move or rename modules.
- Examples should stay minimal, runnable, and aligned with the public SDK surface. Keep sync and async examples consistent when both exist.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/sdk/python && pytest`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Demonstrates sync/async parity in the SDK.

### Purpose

Shows that the sync API surface mirrors the async surface, allowing the same patterns to be used in both contexts.

### Key Files

- `sync.py` -- Synchronous parity example (this folder only has sync.py; async patterns are demonstrated in other examples)

### Imports From

- `_bootstrap` for setup helpers
- `codex_app_server.Codex`
