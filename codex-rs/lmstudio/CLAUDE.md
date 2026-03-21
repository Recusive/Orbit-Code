# codex-rs/lmstudio/

Client library for integrating with a local LM Studio server for open-source model support.

## Build & Test
```bash
cargo build -p orbit-code-lmstudio
cargo test -p orbit-code-lmstudio
```

## Architecture

`LMStudioClient` wraps a `reqwest::Client` and base URL to communicate with a local LM Studio instance. The top-level `ensure_oss_ready()` function orchestrates the full setup flow: verify server connectivity, check model availability, download if missing (via the `lms` CLI), and load in the background. The crate probes the server via the `/models` endpoint and triggers model loading via `/responses`.

## Key Considerations
- Model downloads shell out to the `lms` CLI tool (`lms get --yes`) -- the binary is located via `which` with a fallback to `~/.lmstudio/bin/lms`
- Depends on `orbit-code-core` for `Config` and `LMSTUDIO_OSS_PROVIDER_ID` -- changes to provider IDs or config shape in core will affect this crate
- Uses `reqwest` directly (not workspace version) -- check `Cargo.toml` for the pinned version
- Tests use `wiremock` for HTTP mocking
