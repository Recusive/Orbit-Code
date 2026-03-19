# codex-rs/app-server-protocol/src/protocol/

This file applies to `codex-rs/app-server-protocol/src/protocol/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

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

Core protocol type definitions for the app-server JSON-RPC API. Organized into shared types, v1 legacy types, v2 current types, and supporting modules for mapping, serialization, and thread history reconstruction.

### Key Files

| File | Role |
|------|------|
| `mod.rs` | Module declarations. Exposes `common`, `thread_history`, `v1`, `v2` as public; `mappers` and `serde_helpers` as private. |
| `common.rs` | Shared protocol types used by both v1 and v2. Defines `AuthMode` enum, `GitSha` newtype, and the top-level `ClientRequest` and `ServerNotification` tagged enums that dispatch JSON-RPC methods to the appropriate v1 or v2 handler types. Also defines `ServerRequest`, `ClientNotification`, and experimental API method/field registries. |
| `v1.rs` | Legacy v1 protocol types: `InitializeParams`, `InitializeResponse`, `ClientInfo`, `InitializeCapabilities`, `Profile`, `SandboxSettings`, `Tools`, `UserSavedConfig`, `GetAuthStatusParams/Response`, `GetConversationSummaryParams/Response`, `GitDiffToRemoteParams/Response`, etc. |
| `v2.rs` | The bulk of the protocol. Hundreds of request/response/notification structs covering the full v2 API surface: threads, turns, config, plugins, accounts, models, MCP, filesystem, approvals, command execution, analytics, skills, reviews, plans, dynamic tools, and more. |
| `mappers.rs` | `From` implementations for converting between v1 and v2 types (e.g., `v1::ExecOneOffCommandParams` to `v2::CommandExecParams`). |
| `serde_helpers.rs` | Custom serde helpers: `deserialize_double_option` and `serialize_double_option` for `Option<Option<T>>` patterns using `serde_with`. |
| `thread_history.rs` | Types and conversion logic for reconstructing thread item history from core protocol events. Converts `EventMsg` variants into `ThreadItem` / `Turn` structures. |

### What It Plugs Into

- Re-exported by the parent `lib.rs` via `pub use protocol::common::*;`, `pub use protocol::v1::...;`, `pub use protocol::v2::*;`, `pub use protocol::thread_history::*;`.
- Types derive `Serialize`, `Deserialize`, `JsonSchema`, and `TS` for JSON Schema and TypeScript generation.

### Imports From

- `codex-protocol` -- Core types: `ThreadId`, `SessionSource`, `AskForApproval`, `SandboxPolicy`, `FileChange`, `ResponseItem`, `EventMsg`, `ReasoningEffort`, `ParsedCommand`, and many more.
- `codex-experimental-api-macros` -- `#[derive(ExperimentalApi)]` for experimental gating.
- `codex-utils-absolute-path` -- `AbsolutePathBuf`.
- `schemars`, `ts-rs`, `serde`, `serde_json`, `serde_with` -- Schema generation and serialization.

### Exports To

- Everything flows through `lib.rs` to all consumers of the `codex-app-server-protocol` crate.
