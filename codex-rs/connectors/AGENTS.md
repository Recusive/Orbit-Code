# codex-rs/connectors/

This file applies to `codex-rs/connectors/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-connectors` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-connectors`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Connector (third-party app integration) listing, caching, and merging logic.

### What this folder does

Manages the retrieval and caching of connector (app) information from the ChatGPT directory API. Provides in-memory caching with TTL, connector listing with workspace vs personal account awareness, and response deserialization for the directory listing endpoint.

### Where it plugs in

- Used by `codex-chatgpt::connectors` for fetching and caching connector lists
- Uses `codex-app-server-protocol` for `AppInfo`, `AppBranding`, `AppMetadata` types
- The `list_all_connectors_with_options` function takes a generic async fetch callback, decoupling it from the HTTP client

### Imports from

- `codex-app-server-protocol` -- `AppInfo`, `AppBranding`, `AppMetadata` types
- `serde` -- deserialization of directory API responses
- `urlencoding` -- URL encoding for directory paths

### Exports to

Public API from `lib.rs`:

- `AllConnectorsCacheKey` -- cache key combining base URL, account ID, user ID, and workspace flag
- `DirectoryListResponse` -- deserialization type for the directory API
- `list_all_connectors_with_options` -- fetches and caches connectors with a generic fetch callback
- `cached_all_connectors` -- returns cached connectors if valid
- `CONNECTORS_CACHE_TTL` -- 1-hour TTL constant

### Key files

| File | Role |
|------|------|
| `Cargo.toml` | Crate manifest; depends on `codex-app-server-protocol`, `serde`, `urlencoding` |
| `src/lib.rs` | Full implementation: `AllConnectorsCacheKey`, `CachedAllConnectors`, global `ALL_CONNECTORS_CACHE` static, `DirectoryListResponse` deserialization, `list_all_connectors_with_options`, `cached_all_connectors` |
