# sdk/python-runtime/src/

Source root for the `orbit-code-cli-bin` Python package.

## Purpose

Contains the single package `orbit_code_cli_bin/` which is the installable Python module for the Codex CLI runtime distribution.

## Contents

- `orbit_code_cli_bin/` -- the Python package providing `bundled_codex_path()`

## Plugs Into

- Referenced by `pyproject.toml` as `[tool.hatch.build.targets.wheel] packages = ["src/orbit_code_cli_bin"]`
