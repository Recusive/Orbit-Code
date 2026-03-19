# codex-rs/lmstudio/

Client library for integrating Codex with a local LM Studio server.

## What this folder does

Provides the `codex-lmstudio` crate, which manages communication with a local LM Studio instance for running open-source models. It handles server connectivity checks, model listing, model downloading (via the `lms` CLI), and model loading.

## Where it plugs in

- **Consumed by**: `codex-cli` and `codex-tui` when the `--oss` flag is used and the configured provider is LM Studio.
- **Depends on**: `codex-core` (for `Config`, `LMSTUDIO_OSS_PROVIDER_ID`, and provider definitions), `reqwest` (HTTP client), `which` (finding `lms` binary), `serde_json`, `tokio`, `tracing`.

## Main functions

- `ensure_oss_ready(config)` -- Top-level entry point: verifies the LM Studio server is reachable, checks if the requested model is available, downloads it if missing, and loads it in the background.
- `LMStudioClient::try_from_provider(config)` -- Constructs a client from the configured provider, validates server connectivity.
- `LMStudioClient::fetch_models()` -- Lists available models via the `/models` endpoint.
- `LMStudioClient::download_model(model)` -- Downloads a model using the `lms` CLI tool.
- `LMStudioClient::load_model(model)` -- Triggers model loading via the `/responses` endpoint.

## Key files

| File | Role |
|------|------|
| `Cargo.toml` | Crate manifest; depends on codex-core, reqwest, which |
| `src/lib.rs` | Public API: `ensure_oss_ready()`, `DEFAULT_OSS_MODEL`, re-exports `LMStudioClient` |
| `src/client.rs` | `LMStudioClient` implementation: server health check, model fetch/download/load, `lms` binary discovery with platform-specific fallback paths |
