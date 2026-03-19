# codex-rs/core/tests/common/

This file applies to `codex-rs/core/tests/common/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `core_test_support` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p core_test_support`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Shared test utility library (`core_test_support`) for `codex-core` integration tests.

### What this folder does

Provides reusable test infrastructure used by all integration tests in the `suite/` directory. This is a separate Cargo library crate (`core_test_support`) that integration tests depend on.

### Key files

| File | Purpose |
|------|---------|
| `lib.rs` | Main library entry point; sets up deterministic test mode, configures insta workspace root |
| `test_codex.rs` | Test session creation helpers for `CodexThread` with mock servers |
| `test_codex_exec.rs` | Exec-mode test helpers |
| `responses.rs` | Helpers for constructing mock OpenAI API responses |
| `streaming_sse.rs` | SSE (Server-Sent Events) stream construction for mock API responses |
| `process.rs` | Test process management utilities |
| `context_snapshot.rs` | Context snapshot helpers for verifying model-visible layout |
| `apps_test_server.rs` | Mock MCP server for testing app/connector integration |
| `tracing.rs` | Test tracing configuration |
| `zsh_fork.rs` | Zsh-fork backend test utilities |

### Key setup (via `ctor`)

- `set_thread_manager_test_mode(true)` -- enables deterministic thread IDs
- `set_deterministic_process_ids(true)` -- enables reproducible unified exec process IDs
- Configures `INSTA_WORKSPACE_ROOT` for snapshot test discovery

### Imports from

- `codex_core` -- All public API types for testing
- `codex_utils_cargo_bin` -- Repository root detection
- `wiremock` -- HTTP server mocking
- `tempfile` -- Temporary test directories

### Exports to

- `codex-rs/core/tests/suite/` -- All integration test modules depend on this library
