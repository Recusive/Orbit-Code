# codex-rs/utils/image/

This file applies to `codex-rs/utils/image/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-utils-image` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-utils-image`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Crate `codex-utils-image` -- image processing for LLM prompts.

### What this folder does

Loads, validates, optionally resizes, and encodes images for inclusion in LLM prompts. Supports PNG, JPEG, GIF, and WebP formats. Uses a global LRU cache keyed by content SHA-1 to avoid reprocessing identical files.

### Key types and functions

- `EncodedImage` -- struct holding encoded bytes, MIME type, width, and height; has `into_data_url()` for base64 data URLs
- `PromptImageMode` -- enum: `ResizeToFit` (max 2048x768) or `Original`
- `load_for_prompt_bytes(path, file_bytes, mode) -> Result<EncodedImage, ImageProcessingError>` -- main entry point; auto-detects format, optionally resizes, preserves source bytes when possible
- `MAX_WIDTH` / `MAX_HEIGHT` -- 2048x768 resize constraints

### Imports from

- `base64` -- base64 encoding for data URLs
- `image` -- decoding, resizing, and encoding (PNG, JPEG, GIF, WebP)
- `codex-utils-cache` -- `BlockingLruCache` and `sha1_digest` for content-addressed caching
- `mime_guess` -- MIME type detection from file paths
- `thiserror` -- error type derivation

### Exports to

Consumed by `codex-core` when preparing image content for model requests.

### Key files

- `Cargo.toml` -- crate metadata and dependencies
- `src/lib.rs` -- `EncodedImage`, `PromptImageMode`, `load_for_prompt_bytes`, image encoding/resizing logic, global `IMAGE_CACHE`, and tests
- `src/error.rs` -- `ImageProcessingError` enum with variants for read, decode, encode, and unsupported format errors
