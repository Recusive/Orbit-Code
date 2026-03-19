# sdk/python-runtime/

Template package for `codex-cli-bin` -- the platform-specific Codex CLI runtime distribution for Python.

## Purpose

This is the packaging template for distributing the Codex CLI binary as a Python wheel. The published `codex-app-server-sdk` pins an exact `codex-cli-bin` version, and that runtime package carries the platform-specific `codex` binary for the target wheel platform.

This package is **wheel-only** -- building an sdist is intentionally blocked by the custom build hook.

## Key Files

| File | Role |
|------|------|
| `pyproject.toml` | Package metadata for `codex-cli-bin`; version `0.0.0-dev` (set during release staging); declares custom build hook |
| `hatch_build.py` | Custom Hatch build hook: blocks sdist builds and marks the wheel as platform-specific (`pure_python=False`, `infer_tag=True`) |
| `README.md` | Brief description of the package's purpose |
| `src/codex_cli_bin/__init__.py` | Provides `bundled_codex_path()` function to locate the bundled binary |

## Imports From

- Nothing external (self-contained package)

## Exports To

- `codex_app_server.client` imports `from codex_cli_bin import bundled_codex_path` to resolve the CLI binary path
- The `_runtime_setup.py` in `sdk/python/` stages this package template with a real binary during release

## Binary Location

At runtime, the binary is expected at `<package_dir>/bin/codex` (or `codex.exe` on Windows). The binary is placed there during release staging by `sdk/python/scripts/update_sdk_artifacts.py stage-runtime`.
