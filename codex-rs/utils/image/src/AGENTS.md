# codex-rs/utils/image/src/

This file applies to `codex-rs/utils/image/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-utils-image` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-utils-image`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source directory for the `codex-utils-image` crate.

### Key files

- `lib.rs` -- main implementation containing:
  - `EncodedImage` struct with `into_data_url()` method
  - `PromptImageMode` enum: `ResizeToFit` or `Original`
  - `IMAGE_CACHE` -- global `LazyLock<BlockingLruCache>` with capacity 32, keyed by `ImageCacheKey` (SHA-1 digest + mode)
  - `load_for_prompt_bytes` -- loads from bytes, checks cache, detects format via `image::guess_format`, resizes if needed using `FilterType::Triangle`, preserves source bytes for PNG/JPEG/WebP when within bounds
  - `encode_image` -- encodes `DynamicImage` to PNG, JPEG (quality 85), or lossless WebP
  - `can_preserve_source_bytes` -- determines if source bytes can be passed through (PNG, JPEG, WebP only)
  - Tests for within-bounds passthrough, downscaling, original mode, invalid images, and cache invalidation on content change
- `error.rs` -- `ImageProcessingError` enum:
  - `Read` -- file I/O error
  - `Decode` -- image decoding failure
  - `Encode` -- image encoding failure
  - `UnsupportedImageFormat` -- unrecognized MIME type
  - `decode_error` helper and `is_invalid_image` predicate
