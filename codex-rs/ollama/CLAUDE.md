# codex-rs/ollama/

Client library for integrating with a local Ollama server for open-source model support.

## Build & Test
```bash
cargo build -p orbit-code-ollama
cargo test -p orbit-code-ollama
```

## Architecture

`OllamaClient` wraps a `reqwest::Client` and communicates with the Ollama server using both its native API (`/api/tags`, `/api/version`, `/api/pull`) and OpenAI-compatible endpoints (`/v1/models`). The client auto-detects which API style to use based on the base URL. The top-level `ensure_oss_ready()` orchestrates server verification, model availability checks, and streaming model pulls with progress reporting. `ensure_responses_supported()` validates the Ollama version is >= 0.13.4 for Responses API compatibility.

## Key Considerations
- `wiremock` is a regular dependency (not just dev), because it is used in the library for test support in downstream crates
- Ollama version `0.0.0` (dev builds) is treated as compatible with the Responses API
- The URL module detects `/v1` suffix to distinguish OpenAI-compatible vs native Ollama endpoints -- `base_url_to_host_root()` strips `/v1` to reach native APIs
- Pull progress reporting uses `async-stream` for streaming events -- `PullProgressReporter` trait allows TUI and CLI to render differently
- `DEFAULT_OSS_MODEL` is `"gpt-oss:20b"`
