# codex-rs/app-server-protocol/

This file applies to `codex-rs/app-server-protocol/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-app-server-protocol` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-app-server-protocol`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just write-app-server-schema`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### Purpose

The `codex-app-server-protocol` crate defines the complete JSON-RPC protocol for communication between the app-server and its clients. It contains all request, response, notification, and error types for both v1 and v2 protocol versions, along with JSON Schema and TypeScript type generation tooling.

This is the single source of truth for the app-server API contract. Schema fixtures (JSON and TypeScript) are generated from Rust types and committed to the `schema/` directory for cross-language consumers.

### What It Plugs Into

- **Consumed by:** `codex-app-server`, `codex-app-server-client`, `codex-app-server-test-client`, TUI, exec surface, and any other crate that sends or receives app-server messages.
- **Depends on:** `codex-protocol` for lower-level shared types, `codex-utils-absolute-path`, `codex-experimental-api-macros` for experimental API gating.

### Key Exports

- **JSON-RPC primitives:** `JSONRPCMessage`, `JSONRPCRequest`, `JSONRPCResponse`, `JSONRPCNotification`, `JSONRPCError`, `RequestId`, `Result`.
- **Typed enums:** `ClientRequest`, `ClientNotification`, `ServerRequest`, `ServerNotification` -- tagged unions for all protocol methods.
- **v1 types:** `InitializeParams`, `InitializeResponse`, `ClientInfo`, `InitializeCapabilities`, etc.
- **v2 types:** Hundreds of request/response/notification structs covering threads, turns, config, plugins, MCP, accounts, models, filesystem, etc.
- **Schema generation:** `generate_json`, `generate_ts`, `write_schema_fixtures`, `GenerateTsOptions`.
- **Experimental API:** `ExperimentalApi` trait, `experimental_fields()`, `experimental_required_message()`.
- **Thread history:** Types for reconstructing thread item history from events.

### Key Files

| File | Role |
|------|------|
| `Cargo.toml` | Crate manifest |
| `src/lib.rs` | Crate root; re-exports all protocol types and schema generation APIs |
| `src/protocol/` | Protocol type definitions (common, v1, v2, mappers, serde_helpers, thread_history) |
| `src/jsonrpc_lite.rs` | JSON-RPC envelope types (message, request, response, error, notification) |
| `src/export.rs` | JSON Schema and TypeScript generation logic |
| `src/schema_fixtures.rs` | Schema fixture read/write/comparison utilities |
| `src/experimental_api.rs` | Experimental API gating trait and field registry |
| `src/bin/write_schema_fixtures.rs` | CLI binary to regenerate schema fixtures |
| `schema/` | Generated schema fixtures (JSON and TypeScript) |
| `tests/schema_fixtures.rs` | Tests ensuring committed fixtures match generated output |

### Imports From

- `codex-protocol` -- Core shared types (ThreadId, SessionSource, events, config types, models).
- `codex-experimental-api-macros` -- Proc macro for `#[derive(ExperimentalApi)]`.
- `codex-utils-absolute-path` -- `AbsolutePathBuf`.
- `schemars` -- JSON Schema derivation.
- `ts-rs` -- TypeScript type generation.
- `serde`, `serde_json` -- Serialization.

### Exports To

- Used by every crate that communicates with the app-server.
