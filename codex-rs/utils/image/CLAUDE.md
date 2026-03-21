# codex-rs/utils/image/

Image loading, validation, optional resizing (max 2048x768), and base64 encoding for LLM prompts. Supports PNG, JPEG, GIF, and WebP with a global LRU cache keyed by content SHA-1.

## Build & Test
```bash
cargo build -p orbit-code-utils-image
cargo test -p orbit-code-utils-image
```

## Key Considerations
- Preserves source bytes when possible (PNG/JPEG/WebP within size bounds) to avoid re-encoding artifacts.
- The global `IMAGE_CACHE` requires a multi-threaded Tokio runtime (via `orbit-code-utils-cache`).
