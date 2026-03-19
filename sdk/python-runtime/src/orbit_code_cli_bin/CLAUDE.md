# sdk/python-runtime/src/codex_cli_bin/

The `codex_cli_bin` Python package -- provides a function to locate the bundled Codex CLI binary.

## Purpose

Single-purpose package that exposes `bundled_codex_path()` which returns the `Path` to the platform-specific `codex` binary bundled inside this package's `bin/` directory.

## Key Files

- `__init__.py` -- Defines `bundled_codex_path()` and `PACKAGE_NAME` constant

## Main Function

`bundled_codex_path() -> Path`: Returns the path to `<package>/bin/codex` (or `codex.exe` on Windows). Raises `FileNotFoundError` if the binary is missing.

## Imports From

- Python standard library only (`os`, `pathlib`)

## Exports To

- `codex_app_server.client` calls `from codex_cli_bin import bundled_codex_path` for automatic binary resolution
- Public API: `PACKAGE_NAME`, `bundled_codex_path`
