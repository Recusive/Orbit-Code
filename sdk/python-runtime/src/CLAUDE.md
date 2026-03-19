# sdk/python-runtime/src/

Source root for the `codex-cli-bin` Python package.

## Purpose

Contains the single package `codex_cli_bin/` which is the installable Python module for the Codex CLI runtime distribution.

## Contents

- `codex_cli_bin/` -- the Python package providing `bundled_codex_path()`

## Plugs Into

- Referenced by `pyproject.toml` as `[tool.hatch.build.targets.wheel] packages = ["src/codex_cli_bin"]`
