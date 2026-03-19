# codex-rs/protocol/

Core protocol types crate (`codex-protocol`) defining the data structures for Codex CLI sessions, events, configuration, permissions, and models.

## What this folder does

This crate defines the shared type system used across all Codex Rust crates. It contains:
- The session protocol (Submission Queue / Event Queue pattern) for user-agent communication
- Configuration types (sandbox modes, approval policies, collaboration modes, web search, etc.)
- Model definitions (ResponseItem variants, content items, OpenAI model metadata)
- Permission and sandbox policy types (filesystem, network, approval workflows)
- Prompt templates embedded as markdown (base instructions, approval policy instructions, sandbox mode instructions, realtime conversation framing)
- Turn items, user input, message history, MCP types, custom prompts, dynamic tools

## What it plugs into

- Used by nearly every crate in the workspace: `codex-core`, `codex-otel`, `codex-tui`, `codex-exec`, `codex-config`, `codex-app-server`, etc.
- Provides the canonical type definitions for JSON serialization/deserialization of the protocol
- Types also generate TypeScript bindings via `ts-rs` for cross-language integration

## Imports from

- `codex-execpolicy` -- execution policy types
- `codex-git` -- git-related utilities
- `codex-utils-absolute-path` -- `AbsolutePathBuf` for filesystem paths
- `codex-utils-image` -- image handling utilities
- `serde`, `serde_json`, `serde_with` -- serialization
- `schemars` -- JSON Schema generation
- `ts-rs` -- TypeScript type generation
- `uuid` -- unique identifiers
- `strum`, `strum_macros` -- enum string conversions
- `icu_decimal`, `icu_locale_core` -- number formatting with locale support

## Exports to

All types are public and consumed across the workspace. Key exports include:
- `protocol` module -- `Op` (submission queue), `EventMsg` (event queue), `SandboxPolicy`, `AskForApproval`, `ReviewDecision`, `SessionSource`, `W3cTraceContext`, `WritableRoot`, and many more
- `config_types` -- `ReasoningSummary`, `SandboxMode`, `ModeKind`, `CollaborationMode`, `Personality`, `WebSearchMode`, `ServiceTier`, `TrustLevel`, etc.
- `models` -- `ResponseItem`, `ContentItem`, `BaseInstructions`, `PermissionProfile`
- `permissions` -- `FileSystemSandboxPolicy`, `NetworkSandboxPolicy`, `FileSystemAccessMode`, `FileSystemPath`, `FileSystemSpecialPath`
- `items` -- `TurnItem` variants (UserMessage, AgentMessage, Plan, Reasoning, WebSearch, etc.)
- `approvals` -- `ExecApprovalRequestEvent`, `ApplyPatchApprovalRequestEvent`, `ElicitationRequest`, `GuardianAssessmentEvent`
- `user_input` -- `UserInput` (Text, Image, LocalImage)
- `ThreadId`, `mcp`, `custom_prompts`, `dynamic_tools`, `message_history`, `plan_tool`, etc.

## Key files

- `Cargo.toml` -- crate definition with dependencies
- `src/lib.rs` -- module declarations and `ThreadId` re-export
- `src/protocol.rs` -- main protocol types (Op, EventMsg, SandboxPolicy, etc.)
- `src/config_types.rs` -- configuration enums and structs
- `src/models.rs` -- ResponseItem, ContentItem, model definitions
- `src/permissions.rs` -- filesystem/network sandbox policies with resolution logic
- `src/items.rs` -- turn item types
- `src/approvals.rs` -- approval request/response events
- `src/prompts/` -- embedded markdown prompt templates
