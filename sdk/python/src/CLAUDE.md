# sdk/python/src/

Source root for the `codex-app-server-sdk` Python package. Contains the single package `codex_app_server/`.

## Purpose

This directory is the `packages` root in `pyproject.toml` for the Hatch build system. Everything under `codex_app_server/` is what gets distributed in the wheel.

## Contents

- `codex_app_server/` -- the installable Python package (see its own CLAUDE.md for details)

## Plugs Into

- Referenced by `pyproject.toml` at `sdk/python/pyproject.toml` as `[tool.hatch.build.targets.wheel] packages = ["src/codex_app_server"]`
- Tests in `sdk/python/tests/` add this directory to `sys.path` via `conftest.py`
- Examples in `sdk/python/examples/` add this directory to `sys.path` via `_bootstrap.py`
