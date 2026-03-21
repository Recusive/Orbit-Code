# codex-rs/lmstudio/src/

Source for the `orbit-code-lmstudio` crate -- LM Studio HTTP client for open-source model management.

## Module Layout
- **Public API** (`lib.rs`): `ensure_oss_ready()` orchestrating server check, model availability, download, and load; exports `LMStudioClient` and `DEFAULT_OSS_MODEL`
- **Client** (`client.rs`): `LMStudioClient` with `try_from_provider()`, `check_server()`, `fetch_models()`, `load_model()`, `download_model()`, and `find_lms()` for locating the CLI binary with platform-specific fallback paths
