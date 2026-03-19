# sdk/python/examples/08_local_image_and_text/

Demonstrates multimodal input with a local image file and text.

## Purpose

Shows how to use `LocalImageInput` with a local file path for multimodal turns. Uses a programmatically generated PNG from `_bootstrap.temporary_sample_image_path()`.

## Key Files

- `sync.py` -- Synchronous version
- `async.py` -- Async version

## Imports From

- `_bootstrap` for setup helpers and `temporary_sample_image_path`
- `codex_app_server.Codex`, `codex_app_server.LocalImageInput`, `codex_app_server.TextInput`
