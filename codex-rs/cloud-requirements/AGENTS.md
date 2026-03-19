# codex-rs/cloud-requirements/

This file applies to `codex-rs/cloud-requirements/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-cloud-requirements` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-cloud-requirements`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Cloud-hosted configuration requirements loader for Codex Business/Enterprise accounts.

### What this folder does

Fetches `requirements.toml` configuration data from the Codex backend API as an alternative to local filesystem loading. Applies only to ChatGPT Business and Enterprise customers. Implements HMAC-signed caching with TTL, retry logic with exponential backoff, auth recovery on 401 responses, and background cache refresh. Fails closed -- if requirements cannot be loaded for eligible accounts, configuration loading fails rather than proceeding without them.

### Where it plugs in

- Implements the `CloudRequirementsLoader` interface from `codex-core::config_loader`
- Used during Codex configuration loading to enforce workspace-managed policies
- Fetches from `/api/codex/config/requirements` or `/wham/config/requirements` via `codex-backend-client`

### Imports from

- `codex-backend-client` -- `Client` for HTTP requests to the backend
- `codex-core` -- `AuthManager`, `CodexAuth`, `CloudRequirementsLoader`, `ConfigRequirementsToml`, config loader types
- `codex-protocol` -- `PlanType` for account eligibility checks
- `codex-otel` -- metrics emission
- `hmac` / `sha2` / `base64` -- HMAC-SHA256 cache signing and verification
- `chrono` -- timestamps for cache TTL
- `tokio` -- async I/O, timeouts, background refresh task

### Exports to

Public API:

- `cloud_requirements_loader(auth_manager, chatgpt_base_url, codex_home)` -- creates a `CloudRequirementsLoader` that spawns a fetch task and a background refresh task
- `cloud_requirements_loader_for_storage(...)` -- convenience wrapper that creates an `AuthManager` internally

### Key files

| File | Role |
|------|------|
| `Cargo.toml` | Crate manifest; depends on `codex-backend-client`, `codex-core`, `codex-protocol`, `hmac`, `sha2` |
| `src/lib.rs` | Full implementation: `CloudRequirementsService` with fetch/retry/cache/refresh logic; `BackendRequirementsFetcher`; HMAC cache signing; extensive test suite with mock fetchers |
