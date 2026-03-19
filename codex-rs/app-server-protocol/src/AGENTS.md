# codex-rs/app-server-protocol/src/

This file applies to `codex-rs/app-server-protocol/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-app-server-protocol` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-app-server-protocol`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just write-app-server-schema`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### Purpose

Source code for the `codex-app-server-protocol` crate. Defines all JSON-RPC protocol types, schema generation logic, and experimental API gating infrastructure.

### Module Structure

#### Core Modules

| File | Role |
|------|------|
| `lib.rs` | Crate root. Re-exports all public types from submodules: protocol types (common, v1, v2), JSON-RPC primitives, schema generation functions, experimental API traits. |
| `jsonrpc_lite.rs` | JSON-RPC envelope types: `RequestId`, `JSONRPCMessage` (tagged union of Request/Notification/Response/Error), `JSONRPCRequest`, `JSONRPCResponse`, `JSONRPCNotification`, `JSONRPCError`, `JSONRPCErrorError`. Does not require the `"jsonrpc": "2.0"` field. |
| `experimental_api.rs` | `ExperimentalApi` trait for marking experimental methods/fields, `ExperimentalField` struct registered via `inventory` for discovery, `experimental_fields()` and `experimental_required_message()` helpers. |
| `export.rs` | Schema generation engine. `generate_json()` produces JSON Schema files, `generate_ts()` produces TypeScript definitions, `generate_types()` produces both. Handles v1/v2 partitioning, experimental field filtering, and Prettier formatting. |
| `schema_fixtures.rs` | Schema fixture management: `write_schema_fixtures()`, `read_schema_fixture_tree()`, `read_schema_fixture_subtree()`, `SchemaFixtureOptions`. Used by the `write-schema-fixtures` binary and tests. |

#### Protocol Submodules (in `protocol/`)

| File | Role |
|------|------|
| `protocol/mod.rs` | Module declarations for the protocol namespace. |
| `protocol/common.rs` | Shared types used by both v1 and v2: `AuthMode`, `GitSha`, `ClientRequest` and `ServerNotification` tagged enums (dispatching methods to v1 or v2 types). |
| `protocol/v1.rs` | v1 protocol types: `InitializeParams`, `InitializeResponse`, `ClientInfo`, `InitializeCapabilities`, `Profile`, `SandboxSettings`, `Tools`, etc. |
| `protocol/v2.rs` | v2 protocol types: the bulk of the protocol -- hundreds of structs for threads, turns, config, plugins, accounts, models, MCP, filesystem, approvals, etc. |
| `protocol/mappers.rs` | Type conversion implementations between v1 and v2 types (e.g., `From<v1::ExecOneOffCommandParams> for v2::CommandExecParams`). |
| `protocol/serde_helpers.rs` | Custom serde helpers for double-option serialization/deserialization patterns. |
| `protocol/thread_history.rs` | Types and logic for reconstructing thread item history from protocol events. |

#### Binary

| File | Role |
|------|------|
| `bin/write_schema_fixtures.rs` | CLI binary (`write-schema-fixtures`) that regenerates `schema/` directory fixtures. Accepts `--schema-root`, `--prettier`, and `--experimental` flags. |

### Imports From

- `codex-protocol` -- Core shared types (ThreadId, SessionSource, events, config types, models, approvals, etc.).
- `codex-experimental-api-macros` -- `#[derive(ExperimentalApi)]` proc macro.
- `codex-utils-absolute-path` -- `AbsolutePathBuf`.
- `schemars` -- JSON Schema generation via `#[derive(JsonSchema)]`.
- `ts-rs` -- TypeScript type generation via `#[derive(TS)]`.
- `serde`, `serde_json`, `serde_with` -- Serialization framework.
- `strum_macros` -- `#[derive(Display)]` for enum string representation.

### Exports To

- All crates that interact with the app-server protocol: `codex-app-server`, `codex-app-server-client`, `codex-app-server-test-client`, TUI, exec, tests.
