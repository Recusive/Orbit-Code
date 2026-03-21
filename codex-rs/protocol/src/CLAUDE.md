# protocol/src

## Module Categories

**Session protocol:** `protocol.rs` -- `Op` (submission queue) and `EventMsg` (event queue), plus `SandboxPolicy`, `AskForApproval`, `ReviewDecision`, `SessionSource`, and supporting types. This is the largest file (~150K) and the central interface.

**Config types:** `config_types.rs` -- enums and structs mirroring TOML config fields (`SandboxMode`, `ModeKind`, `CollaborationMode`, `Personality`, `WebSearchMode`, `ServiceTier`, `TrustLevel`).

**Model types:** `models.rs` -- `ResponseItem`, `ContentItem`, `BaseInstructions`, `PermissionProfile`. `openai_models.rs` -- `ReasoningEffort` and model metadata.

**Permission types:** `permissions.rs` -- `FileSystemSandboxPolicy`, `NetworkSandboxPolicy`, path resolution logic. Contains significant runtime validation, not just type definitions.

**Turn/event types:** `items.rs` (turn items), `approvals.rs` (approval request events), `user_input.rs` (text/image input), `mcp.rs` (MCP tool/resource types), `dynamic_tools.rs` (dynamic tool specs and calls), `message_history.rs` (conversation history).

**Supporting:** `thread_id.rs` (UUID-based `ThreadId`), `custom_prompts.rs`, `plan_tool.rs`, `parse_command.rs`, `num_format.rs`, `memory_citation.rs`, `account.rs`, `request_permissions.rs`, `request_user_input.rs`.

**Prompt templates:** `prompts/` -- markdown files embedded at compile time via `include_str!`. Organized into `base_instructions/`, `permissions/approval_policy/`, `permissions/sandbox_mode/`, and `realtime/`.
