# codex-rs/ollama/src/

Source for the `orbit-code-ollama` crate -- Ollama HTTP client with streaming model pull support.

## Module Layout
- **Public API** (`lib.rs`): `ensure_oss_ready()`, `ensure_responses_supported()`, version support logic; exports `OllamaClient`, `DEFAULT_OSS_MODEL`, pull types, and progress reporters
- **Client** (`client.rs`): `OllamaClient` with server probing (native vs OpenAI-compat), model listing, version fetching, streaming pull, and progress-reported pull
- **Pull system** (`pull.rs`, `parser.rs`): `PullEvent` enum (Status, ChunkProgress, Success, Error); `PullProgressReporter` trait; `CliProgressReporter` with speed/percentage on stderr; JSON stream parsing
- **URL utilities** (`url.rs`): `is_openai_compatible_base_url()` and `base_url_to_host_root()` for API endpoint detection
