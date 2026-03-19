# sdk/python/notebooks/

This file applies to `sdk/python/notebooks/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- This subtree belongs to the `codex-app-server-sdk` Python package. Keep import surfaces and packaging metadata consistent when you move or rename modules.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/sdk/python && pytest`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Jupyter notebook walkthrough for the Python SDK.

### Purpose

Provides an interactive notebook-based tutorial for exploring the SDK. The notebook covers the same ground as the examples but in an exploratory, cell-by-cell format suitable for learning.

### Key Files

- `sdk_walkthrough.ipynb` -- Complete Jupyter walkthrough covering SDK initialization, thread management, turn execution, streaming, and result inspection

### Imports From

- `codex_app_server` (the SDK package)
- Bootstrap helpers similar to `examples/_bootstrap.py`

### Plugs Into

- Referenced from `sdk/python/README.md` as the "Jupyter walkthrough notebook"
