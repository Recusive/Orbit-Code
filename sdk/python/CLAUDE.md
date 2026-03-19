# sdk/python/

Python SDK for the Codex `app-server` JSON-RPC v2 protocol. Published as `codex-app-server-sdk`.

## Purpose

Provides synchronous and async Python clients that spawn `codex app-server --listen stdio://` and communicate via JSON-RPC v2 over stdin/stdout. Exposes high-level `Codex`, `Thread`, and `TurnHandle` abstractions, plus generated Pydantic models for all wire types.

## Key Files

| File | Role |
|------|------|
| `src/codex_app_server/__init__.py` | Package root; re-exports all public symbols |
| `src/codex_app_server/api.py` | High-level SDK surface: `Codex`, `AsyncCodex`, `Thread`, `AsyncThread`, `TurnHandle`, `AsyncTurnHandle`, `RunResult` |
| `src/codex_app_server/client.py` | `AppServerClient` -- synchronous JSON-RPC transport over subprocess stdio; `AppServerConfig` for launch options |
| `src/codex_app_server/async_client.py` | `AsyncAppServerClient` -- async wrapper using `asyncio.to_thread` over the sync client |
| `src/codex_app_server/_run.py` | `RunResult` dataclass and helpers to collect stream events into a completed turn |
| `src/codex_app_server/_inputs.py` | Input type definitions (`TextInput`, `ImageInput`, `LocalImageInput`, etc.) and wire serialization |
| `src/codex_app_server/errors.py` | Exception hierarchy: `AppServerError`, `JsonRpcError`, `ServerBusyError`, etc. |
| `src/codex_app_server/models.py` | Shared Pydantic models: `InitializeResponse`, `Notification`, `ServerInfo`, JSON type aliases |
| `src/codex_app_server/retry.py` | `retry_on_overload` helper with exponential backoff for transient server errors |
| `_runtime_setup.py` | Downloads and installs the pinned `codex-cli-bin` runtime from GitHub releases |
| `pyproject.toml` | Package metadata; version 0.2.0; depends on `pydantic>=2.12` |

## Imports From

- `codex-cli-bin` package (from `sdk/python-runtime/`) -- provides `bundled_codex_path()` to locate the binary
- `pydantic` -- all wire-format models are Pydantic BaseModel subclasses
- Generated types from `src/codex_app_server/generated/v2_all.py` (auto-generated from the Rust JSON schema)

## Exports To

- Consumers install `codex-app-server-sdk` and import from `codex_app_server`
- Primary public API: `Codex`, `AsyncCodex`, `Thread`, `AsyncThread`, `RunResult`

## Build / Dev Commands

```bash
python -m pip install -e .          # editable install
python -m pip install -e ".[dev]"   # with dev deps (pytest, ruff, datamodel-code-generator)
pytest                              # run tests
python scripts/update_sdk_artifacts.py generate-types  # regenerate Pydantic models from JSON schema
```

## Subdirectories

- `src/codex_app_server/` -- package source
- `src/codex_app_server/generated/` -- auto-generated Pydantic models
- `docs/` -- getting-started guide, API reference, FAQ
- `examples/` -- 14 numbered runnable example scripts (sync + async)
- `notebooks/` -- Jupyter walkthrough notebook
- `scripts/` -- `update_sdk_artifacts.py` for codegen and release staging
- `tests/` -- pytest test suite
