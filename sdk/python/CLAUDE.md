# sdk/python/

Python SDK (`orbit-code-app-server-sdk`) providing sync and async clients that spawn `codex app-server --listen stdio://` and communicate via JSON-RPC v2 over stdin/stdout.

## Build & Test

```bash
pip install -e .                # editable install
pip install -e ".[dev]"         # with dev deps (pytest, ruff, datamodel-code-generator)
pytest -q                       # run tests
python scripts/update_sdk_artifacts.py generate-types   # regenerate Pydantic models from JSON schema
```

## Architecture

The SDK exposes `Codex`/`AsyncCodex` as the main entry points. Each creates `Thread`/`AsyncThread` instances that communicate with the `codex app-server` subprocess via `AppServerClient` (sync JSON-RPC transport over stdio). All wire-format models are Pydantic v2 `BaseModel` subclasses, with generated types in `src/orbit_code_app_server/generated/` auto-produced from the Rust JSON schema at `codex-rs/app-server-protocol/schema/json/`.

## Key Considerations

- Requires Python >=3.10 and `pydantic>=2.12`.
- The `orbit-code-cli-bin` runtime package (from `sdk/python-runtime/`) provides `bundled_codex_path()` for binary resolution.
- Generated models in `generated/v2_all.py` are auto-generated -- do not edit by hand. Regenerate with `update_sdk_artifacts.py generate-types`.
- `_runtime_setup.py` handles downloading and installing the pinned runtime from GitHub releases.
- `retry.py` implements exponential backoff for transient server errors (429, 5xx).
