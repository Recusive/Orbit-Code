# sdk/python-runtime/

Template package for `orbit-code-cli-bin` -- distributes the platform-specific `codex` CLI binary as a Python wheel.

## Build & Test

This package is not built locally in normal development. It is a template that gets populated with a real binary during release staging via `sdk/python/scripts/update_sdk_artifacts.py stage-runtime`.

```bash
pip install -e .    # only works after a binary has been staged to bin/
```

## Architecture

The package provides a single function `bundled_codex_path()` that returns the path to the bundled `codex` binary at `<package_dir>/bin/codex` (or `codex.exe` on Windows). The custom Hatch build hook in `hatch_build.py` blocks sdist builds and marks wheels as platform-specific (`pure_python=False`, `infer_tag=True`).

## Key Considerations

- Wheel-only -- building an sdist is intentionally blocked.
- The version is `0.0.0-dev` in the template and gets set to a real version during release staging.
- The `orbit-code-app-server-sdk` Python SDK pins an exact version of this package as a dependency.
- The binary at `bin/codex` does not exist in the source tree; it is placed there by the release pipeline.
