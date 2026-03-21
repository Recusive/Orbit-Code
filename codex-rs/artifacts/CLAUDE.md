# codex-rs/artifacts/

Locates, validates, downloads, and executes the pinned artifact runtime (`@oai/artifact-tool`) for artifact generation. Wraps `orbit-code-package-manager` to manage cached runtime installations under `~/.codex/packages/artifacts/`.

## Build & Test
```bash
cargo build -p orbit-code-artifacts
cargo test -p orbit-code-artifacts
```

## Key Considerations
- Downloads release assets from GitHub releases; requires a JS runtime (Node or Electron) on PATH.
- Integration tests are compiled only on non-Windows platforms.
