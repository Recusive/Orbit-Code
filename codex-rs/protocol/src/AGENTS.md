# codex-rs/protocol/src/

This file applies to `codex-rs/protocol/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-protocol` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-protocol`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source code for the `codex-protocol` crate -- core type definitions for the Codex CLI protocol.

### What this folder does

Contains all Rust source modules that define the session protocol, configuration types, model types, permission types, prompt templates, and supporting utilities.

### Module structure

- `lib.rs` -- crate root; declares all public modules and re-exports `ThreadId`
- `protocol.rs` -- primary protocol types: `Op` (submission queue entries), `EventMsg` (event queue entries), `SandboxPolicy`, `AskForApproval`, `ReviewDecision`, `SessionSource`, `W3cTraceContext`, `WritableRoot`, `FileChange`, `NetworkAccess`, `ReadOnlyAccess`, and many more. Also re-exports approval and permission types.
- `config_types.rs` -- configuration enums/structs: `ReasoningSummary`, `Verbosity`, `SandboxMode`, `ApprovalsReviewer`, `Personality`, `WebSearchMode`, `WebSearchToolConfig`, `ServiceTier`, `TrustLevel`, `AltScreenMode`, `ModeKind`, `CollaborationMode`, `CollaborationModeMask`
- `models.rs` -- `ResponseItem` (Message, Reasoning, FunctionCall, LocalShellCall, WebSearchCall, etc.), `ContentItem`, `BaseInstructions`, `PermissionProfile`, `MessagePhase`, `WebSearchAction`
- `permissions.rs` -- `FileSystemSandboxPolicy`, `NetworkSandboxPolicy`, `FileSystemAccessMode`, `FileSystemPath`, `FileSystemSpecialPath`, `FileSystemSandboxEntry`; includes resolution logic for path-based access control and legacy policy conversion
- `items.rs` -- `TurnItem` variants: `UserMessageItem`, `AgentMessageItem`, `PlanItem`, `ReasoningItem`, `WebSearchItem`, `ImageGenerationItem`, `ContextCompactionItem`; each has `as_legacy_event()` conversion
- `approvals.rs` -- `ExecApprovalRequestEvent`, `ApplyPatchApprovalRequestEvent`, `ElicitationRequest`, `ElicitationRequestEvent`, `GuardianAssessmentEvent`, `ExecPolicyAmendment`, `NetworkApprovalContext`, `NetworkPolicyAmendment`
- `user_input.rs` -- `UserInput` enum (Text, Image, LocalImage), `TextElement`, `ByteRange`
- `custom_prompts.rs` -- `CustomPrompt` struct with name, path, content, description, argument hint
- `dynamic_tools.rs` -- `DynamicToolSpec`, `DynamicToolCallRequest`, `DynamicToolResponse`
- `mcp.rs` -- MCP (Model Context Protocol) types: `Tool`, `Resource`, `ResourceTemplate`, `CallToolResult`, `RequestId`
- `message_history.rs` -- `HistoryEntry` for conversation history persistence
- `openai_models.rs` -- `ReasoningEffort` and OpenAI model metadata
- `parse_command.rs` -- `ParsedCommand` for shell command parsing
- `plan_tool.rs` -- `UpdatePlanArgs` for the plan/TODO tool
- `request_permissions.rs` / `request_user_input.rs` -- permission request and user input request events
- `account.rs` -- account-related types
- `thread_id.rs` -- `ThreadId` (UUID-based conversation identifier)
- `num_format.rs` -- locale-aware number formatting with separators
- `memory_citation.rs` -- `MemoryCitation` for memory-linked responses
- `prompts/` -- embedded markdown prompt templates

### Imports from

- `codex-execpolicy`, `codex-git`, `codex-utils-absolute-path`, `codex-utils-image`
- `serde`, `serde_json`, `serde_with`, `schemars`, `ts-rs`, `uuid`, `strum`, `strum_macros`, `tracing`

### Exports to

All modules are `pub` and consumed by `codex-core`, `codex-otel`, `codex-tui`, `codex-exec`, `codex-config`, `codex-app-server`, and other workspace crates.
