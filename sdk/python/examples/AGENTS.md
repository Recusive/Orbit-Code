# sdk/python/examples/

This file applies to `sdk/python/examples/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- This subtree belongs to the `codex-app-server-sdk` Python package. Keep import surfaces and packaging metadata consistent when you move or rename modules.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/sdk/python && pytest`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Runnable example scripts demonstrating the Python SDK public API. Each numbered folder contains `sync.py` and/or `async.py`.

### Purpose

Provides 14 progressive examples covering the full SDK surface, from basic quickstart to advanced turn controls. All examples use only public exports from `codex_app_server`.

### Key Files

| File | Role |
|------|------|
| `_bootstrap.py` | Shared bootstrap: adds `sdk/python/src` to `sys.path`, ensures the `codex-cli-bin` runtime is installed, provides `runtime_config()` and helper utilities |
| `README.md` | Index of all examples with descriptions and prerequisites |

### Example Index

| Folder | Topic |
|--------|-------|
| `01_quickstart_constructor/` | First run / sanity check |
| `02_turn_run/` | Inspect full turn output fields |
| `03_turn_stream_events/` | Stream a turn with event view |
| `04_models_and_metadata/` | Discover visible models |
| `05_existing_thread/` | Resume an existing thread |
| `06_thread_lifecycle_and_controls/` | Thread lifecycle + control calls |
| `07_image_and_text/` | Remote image URL + text multimodal turn |
| `08_local_image_and_text/` | Local image + text multimodal turn |
| `09_async_parity/` | Async parity example |
| `10_error_handling_and_retry/` | Overload retry + typed error handling |
| `11_cli_mini_app/` | Interactive chat loop |
| `12_turn_params_kitchen_sink/` | Structured output with advanced turn config |
| `13_model_select_and_turn_params/` | List models, pick highest, run turns |
| `14_turn_controls/` | Steer and interrupt demos |

### Imports From

- `codex_app_server` (the SDK package via local `sys.path` injection)
- `_bootstrap.py` for runtime setup and helpers
- `_runtime_setup.py` (from `sdk/python/`) for downloading the CLI binary

### Running

```bash
cd sdk/python
python examples/01_quickstart_constructor/sync.py
python examples/01_quickstart_constructor/async.py
```
