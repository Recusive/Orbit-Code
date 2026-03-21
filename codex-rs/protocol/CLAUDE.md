# protocol

Core shared type definitions for the entire Orbit Code workspace.

## Build & Test

```bash
cargo test -p orbit-code-protocol          # Run tests
just fix -p orbit-code-protocol            # Clippy
just fmt                                    # Format
```

## Architecture

This crate defines the shared type system consumed by nearly every other crate in the workspace. There is no runtime logic here -- it is purely types, serialization, and embedded prompt templates.

### Core Abstraction: SQ/EQ Protocol

The central pattern is a Submission Queue / Event Queue protocol defined in `protocol.rs`:

- **`Op`** (Submission Queue) -- commands the client sends to the agent: `UserTurn`, `ConfigureSession`, `ReviewApproval`, etc.
- **`EventMsg`** (Event Queue) -- events the agent emits back: `Turn`, `GetInput`, `AgentError`, `TaskComplete`, etc.

These two enums are the fundamental interface between the TUI/app-server and the core agent runtime.

### Type Categories

The remaining modules fall into four groups:

1. **Config types** (`config_types.rs`) -- `SandboxMode`, `ModeKind`, `CollaborationMode`, `Personality`, `WebSearchMode`, `ServiceTier`, etc. These mirror TOML config fields and derive `JsonSchema`.
2. **Model types** (`models.rs`, `openai_models.rs`) -- `ResponseItem`, `ContentItem`, `BaseInstructions`, `PermissionProfile`. The model-level representation of API responses and tool calls.
3. **Permission types** (`permissions.rs`) -- `FileSystemSandboxPolicy`, `NetworkSandboxPolicy`, filesystem path resolution logic. Contains significant validation/resolution logic unlike the other mostly-declarative modules.
4. **Session types** (`items.rs`, `approvals.rs`, `user_input.rs`, `mcp.rs`, `dynamic_tools.rs`) -- turn items, approval events, MCP tool definitions, dynamic tool specs.

### Prompt Templates

`src/prompts/` contains embedded markdown templates organized into subdirectories:

- `base_instructions/` -- default system prompt
- `permissions/approval_policy/` -- per-approval-mode instructions (never, on_failure, unless_trusted, etc.)
- `permissions/sandbox_mode/` -- per-sandbox-mode instructions (read_only, workspace_write, danger_full_access)
- `realtime/` -- conversation framing for realtime sessions

These are included at compile time via `include_str!` in `protocol.rs`.

### TypeScript Bindings

Most types derive `ts_rs::TS` to generate TypeScript definitions. The bindings are consumed by the app-server-protocol schema generation pipeline.

## Key Considerations

- **Widespread rebuild impact.** This crate is a dependency of nearly every other crate. Any change here triggers rebuilds across the workspace. Keep changes focused.
- **`protocol.rs` is very large** (~150K). When adding new protocol functionality, consider whether it belongs in a separate module (items, approvals, etc.) rather than extending protocol.rs further.
- **`permissions.rs` has runtime logic.** Unlike other modules which are mostly type definitions, permissions.rs contains path resolution and policy validation logic with substantial test coverage.
- **Config types must derive `JsonSchema`.** If you change `config_types.rs`, run `just write-config-schema` to regenerate the config schema.
- **`include_str!` and Bazel.** If you add or rename prompt template files, update the crate's `BUILD.bazel` `compile_data` or Bazel builds will fail.
- **Serde conventions.** Config types use `kebab-case`, protocol types use `snake_case`. Check the existing `rename_all` on the type you are modifying.
