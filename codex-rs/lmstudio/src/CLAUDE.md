# codex-rs/lmstudio/src/

Source directory for the `codex-lmstudio` crate.

## What this folder does

Implements the LM Studio client library that communicates with a local LM Studio server over HTTP. Used when running Codex in open-source model mode with LM Studio as the backend.

## Key files

| File | Purpose |
|------|---------|
| `lib.rs` | Crate entry point. Exports `LMStudioClient`, `DEFAULT_OSS_MODEL` (`"openai/gpt-oss-20b"`), and `ensure_oss_ready()` which orchestrates server verification, model availability check, download, and background loading |
| `client.rs` | Core `LMStudioClient` struct wrapping a `reqwest::Client` and base URL. Methods: `try_from_provider()` (construct from config), `check_server()` (GET `/models`), `fetch_models()` (list available models), `load_model()` (POST `/responses` with minimal payload), `download_model()` (shell out to `lms get --yes`), `find_lms()` (locate the LM Studio CLI with platform-specific fallback to `~/.lmstudio/bin/lms`) |

## Imports / exports

- **Imports**: `codex-core` (`Config`, `LMSTUDIO_OSS_PROVIDER_ID`), `reqwest`, `serde_json`, `tokio`, `tracing`, `which`
- **Exports**: `LMStudioClient`, `ensure_oss_ready()`, `DEFAULT_OSS_MODEL`
