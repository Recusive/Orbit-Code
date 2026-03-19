# codex-rs/docs/

This file applies to `codex-rs/docs/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- Documentation changes should track the current code and command surface. Update examples when behavior or CLI flags change.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex && pnpm format`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Developer and architecture documentation for the codex-rs workspace.

### What this folder does

Contains technical documentation for the internal architecture, protocols, and build systems used by the Rust Codex implementation.

### Key files

- `protocol_v1.md` -- Specification of the core Codex protocol:
  - Defines entities: Model, Codex, Session, Task, Turn
  - Documents the submission queue (SQ) and event queue (EQ) interface between UI and Codex engine
  - Covers `Op` variants (UserTurn, Interrupt, ExecApproval, UserInputAnswer, ListSkills, etc.)
  - Covers `EventMsg` variants (AgentMessage, TurnStarted, TurnComplete, ExecApprovalRequest, etc.)
  - Includes sequence diagrams for basic UI flow and task interruption
  - References `protocol/src/protocol.rs` and `core/src/agent.rs`

- `codex_mcp_interface.md` -- MCP server interface documentation:
  - Describes the JSON-RPC API over Model Context Protocol transport
  - Documents v2 RPCs: thread/start, turn/start, model/list, config/read, etc.
  - Documents legacy v1 compatibility RPCs: getConversationSummary, getAuthStatus, etc.
  - Documents approval flow (applyPatchApproval, execCommandApproval)
  - Documents event streaming and tool response shapes
  - References `app-server-protocol/src/protocol/{common,v1,v2}.rs`

- `bazel.md` -- Bazel build system documentation:
  - Explains the experimental Bazel setup alongside Cargo
  - Documents the `codex_rust_crate` macro from `defs.bzl`
  - Instructions for updating Bzlmod lockfiles and adding new crates

### What it plugs into

- Referenced by developers building integrations against the Codex protocol
- Referenced by the `app-server/` and `protocol/` crates for protocol definitions
- The Bazel docs are referenced when working with `BUILD.bazel` files

### Imports from / exports to

- No code imports; pure documentation
- References source files in `protocol/`, `core/`, `app-server/`, and `app-server-protocol/`
