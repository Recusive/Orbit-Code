# sdk/python/examples/01_quickstart_constructor/

This file applies to `sdk/python/examples/01_quickstart_constructor/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- This subtree belongs to the `codex-app-server-sdk` Python package. Keep import surfaces and packaging metadata consistent when you move or rename modules.
- Examples should stay minimal, runnable, and aligned with the public SDK surface. Keep sync and async examples consistent when both exist.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/sdk/python && pytest`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Quickstart example demonstrating the simplest possible SDK usage.

### Purpose

Shows how to create a `Codex` instance, start a thread, run a single turn, and print the result. This is the recommended first example to run.

### Key Files

- `sync.py` -- Synchronous version using `Codex` context manager
- `async.py` -- Async version using `AsyncCodex`

### Imports From

- `_bootstrap` (parent `examples/` directory) for `ensure_local_sdk_src`, `runtime_config`, `server_label`
- `codex_app_server.Codex` / `codex_app_server.AsyncCodex`

### Running

```bash
cd sdk/python
python examples/01_quickstart_constructor/sync.py
python examples/01_quickstart_constructor/async.py
```
