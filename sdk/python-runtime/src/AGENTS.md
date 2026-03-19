# sdk/python-runtime/src/

This file applies to `sdk/python-runtime/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- This subtree belongs to the `codex-cli-bin` Python package. Keep import surfaces and packaging metadata consistent when you move or rename modules.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/sdk/python-runtime && pytest`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source root for the `codex-cli-bin` Python package.

### Purpose

Contains the single package `codex_cli_bin/` which is the installable Python module for the Codex CLI runtime distribution.

### Contents

- `codex_cli_bin/` -- the Python package providing `bundled_codex_path()`

### Plugs Into

- Referenced by `pyproject.toml` as `[tool.hatch.build.targets.wheel] packages = ["src/codex_cli_bin"]`
