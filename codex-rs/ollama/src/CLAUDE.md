# codex-rs/ollama/src/

Source directory for the `codex-ollama` crate.

## What this folder does

Implements the Ollama client library that communicates with a local Ollama server over HTTP. Supports both native Ollama API and OpenAI-compatible endpoints. Used when running Codex in open-source model mode with Ollama as the backend.

## Key files

| File | Purpose |
|------|---------|
| `lib.rs` | Crate entry point. Exports `OllamaClient`, `DEFAULT_OSS_MODEL` (`"gpt-oss:20b"`), pull types, and progress reporters. `ensure_oss_ready()` orchestrates server verification + model pull. `ensure_responses_supported()` validates Ollama version >= 0.13.4. `supports_responses()` treats version 0.0.0 (dev builds) as compatible |
| `client.rs` | `OllamaClient` struct wrapping `reqwest::Client`, host root URL, and OpenAI-compat flag. Key methods: `try_from_provider()` (construct + probe), `probe_server()` (hits `/api/tags` or `/v1/models`), `fetch_models()` (GET `/api/tags`, extracts model names), `fetch_version()` (GET `/api/version`, parses semver), `pull_model_stream()` (POST `/api/pull` with streaming, yields `PullEvent`s), `pull_with_reporter()` (drives a `PullProgressReporter`) |
| `parser.rs` | `pull_events_from_value()`: converts a single JSON object from the pull stream into `PullEvent` variants. Handles `status` (including `"success"`), `digest`/`total`/`completed` progress fields, and `error` messages |
| `pull.rs` | `PullEvent` enum: `Status(String)`, `ChunkProgress { digest, total, completed }`, `Success`, `Error(String)`. `PullProgressReporter` trait with `on_event()`. `CliProgressReporter`: stderr-based progress with download speed (MB/s), total size (GB), percentage. `TuiProgressReporter`: delegates to CLI reporter |
| `url.rs` | `is_openai_compatible_base_url()`: checks if URL ends with `/v1`. `base_url_to_host_root()`: strips `/v1` suffix to get the native Ollama host root (e.g., `http://localhost:11434/v1` becomes `http://localhost:11434`) |

## Imports / exports

- **Imports**: `codex-core` (`Config`, `ModelProviderInfo`, `OLLAMA_OSS_PROVIDER_ID`, `create_oss_provider_with_base_url`), `reqwest`, `semver`, `async-stream`, `bytes`, `futures`, `serde_json`, `tokio`, `tracing`, `wiremock` (for tests)
- **Exports**: `OllamaClient`, `ensure_oss_ready()`, `ensure_responses_supported()`, `DEFAULT_OSS_MODEL`, `PullEvent`, `PullProgressReporter`, `CliProgressReporter`, `TuiProgressReporter`
