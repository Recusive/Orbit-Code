# sdk/python/src/

Source root for the `orbit-code-app-server-sdk` Python package. Contains the single installable package `orbit_code_app_server/`.

## Module Layout

- **API surface**: `api.py` (Codex, AsyncCodex, Thread, AsyncThread, TurnHandle), `__init__.py` (re-exports)
- **Transport**: `client.py` (sync JSON-RPC over stdio), `async_client.py` (async wrapper via `asyncio.to_thread`)
- **Wire types**: `models.py` (shared Pydantic models), `generated/` (auto-generated from Rust JSON schema -- do not edit)
- **Inputs/outputs**: `_inputs.py` (TextInput, ImageInput, etc.), `_run.py` (RunResult dataclass)
- **Error handling**: `errors.py` (AppServerError, JsonRpcError, ServerBusyError hierarchy)
- **Utilities**: `retry.py` (exponential backoff for transient errors)
