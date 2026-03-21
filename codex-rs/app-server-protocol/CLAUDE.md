# app-server-protocol

Single source of truth for the JSON-RPC API contract between the app-server and its clients. Defines all request, response, notification, and error types for v1 and v2 protocol versions, plus JSON Schema and TypeScript binding generation.

## Build & Test

```bash
cargo test -p orbit-code-app-server-protocol   # Run tests (includes schema fixture validation)
just write-app-server-schema                    # Regenerate schema fixtures in schema/
just write-app-server-schema --experimental     # Include experimental API surface
just fmt                                        # Format
just fix -p orbit-code-app-server-protocol      # Clippy
```

The `schema/` directory contains committed JSON and TypeScript fixtures generated from Rust types. Tests in `tests/schema_fixtures.rs` verify that committed fixtures match generated output -- if you change types, you must regenerate.

## Architecture

### Protocol Structure

The protocol is split across three files in `src/protocol/`:

- **`common.rs`** -- the core dispatch layer. Defines `ClientRequest`, `ClientNotification`, `ServerRequest`, and `ServerNotification` as tagged enums that route JSON-RPC method names to their v1 or v2 handler types. Also defines `AuthMode` and experimental API method registries via macros.
- **`v1.rs`** -- legacy v1 types (`InitializeParams`, `InitializeResponse`, `ClientInfo`, etc.). No new API surface should be added here.
- **`v2.rs`** -- the active v2 API surface. Hundreds of `*Params`/`*Response`/`*Notification` structs covering threads, turns, config, plugins, accounts, models, MCP, filesystem, approvals, command execution, analytics, and more.

### TypeScript & JSON Schema Generation

Types derive `ts_rs::TS` and `schemars::JsonSchema`. The generation pipeline in `export.rs` produces both JSON Schema files and TypeScript definitions, partitioned by v1/v2 and with experimental field filtering. The `bin/write_schema_fixtures.rs` binary drives the regeneration.

### Experimental API Gating

The `ExperimentalApi` trait (from `orbit-code-experimental-api-macros`) marks methods and fields as experimental. The `common.rs` macros register experimental methods, and `#[derive(ExperimentalApi)]` with `#[experimental("reason")]` on struct fields provides field-level gating. Use `inspect_params: true` in the macro when only some fields of a stable method are experimental.

### Supporting Modules

- `jsonrpc_lite.rs` -- JSON-RPC envelope types (`JSONRPCMessage`, `JSONRPCRequest`, `JSONRPCResponse`, `JSONRPCNotification`, `JSONRPCError`). Note: does not require the `"jsonrpc": "2.0"` field.
- `mappers.rs` -- `From` impls converting between v1 and v2 types.
- `serde_helpers.rs` -- custom serde for `Option<Option<T>>` (double-option) patterns.
- `thread_history.rs` -- reconstructing thread item history from `EventMsg` events.

## Key Considerations

- **All new API work goes in v2.** Do not add new types or methods to `v1.rs`.
- **v2 wire format is `camelCase`.** All v2 types use `#[serde(rename_all = "camelCase")]`.
- **`#[ts(optional = nullable)]` is for `*Params` fields only.** Do not use it on response or notification types.
- **Never use `skip_serializing_if` on v2 `Option<T>` fields.** Use `#[ts(optional = nullable)]` instead. Exception: booleans defaulting to false use `#[serde(default, skip_serializing_if = "std::ops::Not::not")]`.
- **Always add `#[ts(export_to = "v2/")]`** on v2 types to ensure TypeScript bindings land in the right directory.
- **Keep `#[serde(rename)]` and `#[ts(rename)]` aligned** when renaming fields for wire compatibility.
- **Regenerate after changes.** Run `just write-app-server-schema` after any type changes, then `cargo test -p orbit-code-app-server-protocol` to verify fixtures match.
- **`v2.rs` is very large** (~270K). Follow the naming conventions (`*Params`, `*Response`, `*Notification`) and group related types together near their RPC method registration in `common.rs`.
- **Experimental fields need `ExperimentalApi`.** If adding an experimental field to a stable method's params, derive `ExperimentalApi` on the params struct, mark the field with `#[experimental("method/name")]`, and set `inspect_params: true` on the method variant in `common.rs`.
