# app-server-protocol/src

## Module Categories

**Core protocol dispatch:** `protocol/common.rs` -- `ClientRequest`, `ClientNotification`, `ServerRequest`, `ServerNotification` tagged enums that route JSON-RPC methods to v1 or v2 types. Also defines `AuthMode`, `GitSha`, and experimental method registries via macros. This is the most architecturally important file.

**API versions:** `protocol/v1.rs` (legacy, frozen -- no new surface), `protocol/v2.rs` (active development -- all new `*Params`/`*Response`/`*Notification` types go here).

**Type conversion:** `protocol/mappers.rs` (`From` impls between v1 and v2 types), `protocol/serde_helpers.rs` (double-option serde for `Option<Option<T>>`).

**Thread history:** `protocol/thread_history.rs` -- reconstructs thread item history from core `EventMsg` events into `ThreadItem`/`Turn` structures.

**JSON-RPC envelope:** `jsonrpc_lite.rs` -- `JSONRPCMessage`, `JSONRPCRequest`, `JSONRPCResponse`, `JSONRPCNotification`, `JSONRPCError`. Does not enforce `"jsonrpc": "2.0"`.

**Experimental API:** `experimental_api.rs` -- `ExperimentalApi` trait, field registry via `inventory`, helper functions for experimental gating.

**Schema generation:** `export.rs` -- produces JSON Schema and TypeScript definitions from Rust types, handles v1/v2 partitioning and experimental field filtering. `schema_fixtures.rs` -- fixture read/write/comparison utilities.

**Binaries:** `bin/write_schema_fixtures.rs` (drives `just write-app-server-schema`), `bin/export.rs`.

**Entry point:** `lib.rs` -- re-exports everything from submodules. Uses `pub use protocol::v2::*` so v2 types are available at crate root.
