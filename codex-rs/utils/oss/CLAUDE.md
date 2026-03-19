# codex-rs/utils/oss/

Crate `codex-utils-oss` -- OSS (open-source) model provider utilities.

## What this folder does

Shared utilities for working with local/OSS model providers (LM Studio and Ollama). Provides functions to get default models and ensure providers are ready, used by both the TUI and exec entry points.

## Key types and functions

- `get_default_model_for_oss_provider(provider_id) -> Option<&str>` -- returns the default model name for LM Studio or Ollama
- `ensure_oss_provider_ready(provider_id, config) -> Result<(), io::Error>` -- async function that ensures the specified provider is reachable and has required models downloaded

## Imports from

- `codex-core` -- `LMSTUDIO_OSS_PROVIDER_ID`, `OLLAMA_OSS_PROVIDER_ID`, `Config`
- `codex-lmstudio` -- `DEFAULT_OSS_MODEL`, `ensure_oss_ready`
- `codex-ollama` -- `DEFAULT_OSS_MODEL`, `ensure_responses_supported`, `ensure_oss_ready`

## Exports to

Consumed by `codex-tui` and `codex-exec` for OSS provider setup and model selection.

## Key files

- `Cargo.toml` -- crate metadata; depends on `codex-core`, `codex-lmstudio`, `codex-ollama`
- `src/lib.rs` -- `get_default_model_for_oss_provider`, `ensure_oss_provider_ready`, and tests
