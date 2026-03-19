# sdk/python/src/codex_app_server/

The core Python package for the Codex App Server SDK. Provides sync and async clients for driving the Codex agent over JSON-RPC v2.

## Purpose

Exposes a typed Python API for spawning `codex app-server`, managing threads and turns, streaming notifications, and collecting results. All wire types are Pydantic models.

## Key Files

| File | Role |
|------|------|
| `__init__.py` | Re-exports all public symbols; defines `__version__` and `__all__` |
| `api.py` | High-level public surface: `Codex` (sync), `AsyncCodex`, `Thread`, `AsyncThread`, `TurnHandle`, `AsyncTurnHandle`, `RunResult` |
| `client.py` | `AppServerClient` -- low-level sync JSON-RPC transport; spawns subprocess, reads/writes JSONL; `AppServerConfig` for binary resolution and launch options |
| `async_client.py` | `AsyncAppServerClient` -- wraps sync client using `asyncio.to_thread` with a transport lock |
| `_run.py` | `RunResult` dataclass; `_collect_run_result` / `_collect_async_run_result` consume notification streams into completed turn results |
| `_inputs.py` | Input type dataclasses (`TextInput`, `ImageInput`, `LocalImageInput`, `SkillInput`, `MentionInput`) and `_to_wire_input` serialization |
| `errors.py` | Exception hierarchy rooted at `AppServerError`; maps JSON-RPC error codes to typed exceptions; `is_retryable_error` predicate |
| `models.py` | Shared Pydantic models: `InitializeResponse`, `ServerInfo`, `Notification`, `NotificationPayload` union type; JSON type aliases |
| `retry.py` | `retry_on_overload` -- exponential backoff with jitter for `ServerBusyError` |
| `py.typed` | PEP 561 marker for typed package |
| `generated/` | Auto-generated Pydantic models from the app-server JSON schema |

## Imports From

- `pydantic` (BaseModel for all wire types)
- `codex_cli_bin` package (optional; provides `bundled_codex_path()` for binary resolution)
- `generated/v2_all.py` for all app-server request/response/notification Pydantic models
- `generated/notification_registry.py` for method-name-to-model mapping

## Exports To

- External consumers via `from codex_app_server import Codex, Thread, RunResult, ...`
- The `__all__` list in `__init__.py` defines the complete public API

## Call Flow

1. `Codex()` creates an `AppServerClient`, calls `start()` (spawns subprocess), then `initialize()` (JSON-RPC handshake)
2. `codex.thread_start(...)` sends `thread/start` RPC, returns a `Thread` bound to the client
3. `thread.run("prompt")` sends `turn/start` RPC, then streams `next_notification()` calls collecting `item/completed` and `turn/completed` events into a `RunResult`
4. For fine-grained control, `thread.turn(...)` returns a `TurnHandle` with `.stream()`, `.steer()`, and `.interrupt()` methods
