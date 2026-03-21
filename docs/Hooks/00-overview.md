# Hooks System — Production Implementation Plan

## Goal

Bring the Orbit Code hooks system to exact parity with Claude Code's hooks system. Every event, every handler type, every decision control pattern, every wire format — identical behavior.

## Reference

Source of truth: `docs/hooks.md` (Claude Code hooks reference, fetched locally).

## Current State

The `codex-rs/hooks/` crate implements 3 of 21 hook events with 1 of 4 handler types:

| Component | Implemented | Total | Gap |
|-----------|-------------|-------|-----|
| Hook events | 3 (SessionStart, UserPromptSubmit, Stop) | 21 | 18 events |
| Handler types | 1 (command, sync only) | 4 (command, http, prompt, agent) | 3 types + async |
| Matcher support | SessionStart only | All tool/agent events | Many events |
| Decision control | Basic (block/allow/context) | Full (updatedInput, updatedPermissions, etc.) | Rich controls |

## Architecture

All hooks share a single execution pipeline:

```
ConfigLayerStack → discovery.rs → ConfiguredHandler[]
                                        ↓
Event fires → dispatcher::select_handlers(event, matcher_input)
                                        ↓
                              matched handlers[]
                                        ↓
                    ┌───────────────────┼───────────────────┐
                    ↓                   ↓                   ↓
              command_runner      http_runner         prompt_runner
              (stdin/stdout)      (POST/response)    (LLM call)
                    ↓                   ↓                   ↓
                    └───────────────────┼───────────────────┘
                                        ↓
                              output_parser::parse_*()
                                        ↓
                              EventOutcome struct
                                        ↓
                         hook_runtime.rs (core integration)
```

## Phase Breakdown

| Phase | Scope | Events | New Infrastructure |
|-------|-------|--------|-------------------|
| [Phase 1](./01-phase1-agentic-loop.md) | Core agentic loop | PreToolUse, PostToolUse, PostToolUseFailure, PermissionRequest | Tool-name matcher in dispatcher, hookSpecificOutput parsing, updatedInput, updatedPermissions |
| [Phase 2](./02-phase2-session-lifecycle.md) | Session & agent lifecycle | SessionEnd, SubagentStart, SubagentStop, StopFailure, Notification | SessionEnd timeout, agent event wiring |
| [Phase 3](./03-phase3-advanced-handlers.md) | Handler types & infrastructure | HTTP hooks, async command hooks, ConfigChange, InstructionsLoaded | http_runner.rs, async hook spawning, env var interpolation |
| [Phase 4](./04-phase4-remaining-events.md) | Remaining events & prompt/agent hooks | PreCompact, PostCompact, WorktreeCreate, WorktreeRemove, TeammateIdle, TaskCompleted, Elicitation, ElicitationResult, prompt hooks, agent hooks | prompt_runner.rs, agent_runner.rs |

## File Impact Summary

### Protocol crate (`codex-rs/protocol/src/protocol.rs`)

`HookEventName` enum grows from 3 → 21 variants. `HookHandlerType` grows to include `Http`. All existing consumers updated.

### Hooks crate (`codex-rs/hooks/`)

| File | Change |
|------|--------|
| `src/engine/config.rs` | `HookEvents` struct: 3 → 21 event fields. `HookHandlerConfig`: add `Http` variant |
| `src/engine/discovery.rs` | 3 → 21 discovery loops. Matcher routing per event type |
| `src/engine/dispatcher.rs` | `select_handlers` and `scope_for_event`: 3 → 21 match arms. Tool-name matcher support |
| `src/engine/command_runner.rs` | No change for sync. New `async_command_runner.rs` for async |
| `src/engine/http_runner.rs` | **New file.** HTTP POST with env var interpolation, response parsing |
| `src/engine/prompt_runner.rs` | **New file.** Single-turn LLM evaluation |
| `src/engine/agent_runner.rs` | **New file.** Multi-turn subagent verification |
| `src/engine/mod.rs` | Add 18 new `run_*`/`preview_*` method pairs |
| `src/engine/output_parser.rs` | Add 18 new `parse_*` functions with typed output structs |
| `src/schema.rs` | Add 18 new `CommandInput`/`CommandOutputWire` struct pairs. `HookEventNameWire`: 3 → 21 variants |
| `src/events/` | **18 new event module files**, each ~200-400 lines with Request, Outcome, run(), preview(), tests |
| `src/registry.rs` | Add 18 new `run_*`/`preview_*` method pairs on `Hooks` |
| `src/types.rs` | Extend `HookEvent` enum for new event payload types |

### Core crate (`codex-rs/core/`)

| File | Change |
|------|--------|
| `src/hook_runtime.rs` | Add integration functions for each new event |
| `src/codex.rs` | Wire PreToolUse/PostToolUse into tool execution flow |
| `src/tools/registry.rs` or `src/tools/router.rs` | Call pre/post tool hooks around tool dispatch |
| `src/agent/` | Wire SubagentStart/SubagentStop hooks |
| Various session lifecycle points | Wire SessionEnd, Notification, ConfigChange, etc. |

### Schema fixtures (`codex-rs/hooks/schema/generated/`)

36 new JSON Schema files (input + output for each of 18 events).

## Conventions

Every new event module follows the exact pattern established by `session_start.rs`, `user_prompt_submit.rs`, and `stop.rs`:

1. `Request` struct with all input fields
2. `Outcome` struct with hook_events + event-specific results
3. Private `HandlerData` struct for per-handler parsed results
4. `preview()` → `Vec<HookRunSummary>`
5. `run()` → `Outcome`
6. `parse_completed()` → `ParsedHandler<HandlerData>`
7. Tests for every exit code path and JSON output variant
8. Schema fixtures for input and output

No shortcuts. No "TODO: implement later". Every wire format, every error path, every test.

---

## Cross-Cutting Concerns

These items span multiple phases and must be addressed alongside the phase they first appear in. They are **not optional** — without them, parity is incomplete.

### 1. `agent_id` / `agent_type` on ALL event inputs (Phase 1+)

Per spec, when a hook fires inside a subagent (or when the session was started with `--agent`), **every** event input gets two additional optional fields:

```rust
// Add to EVERY CommandInput struct as optional fields:
#[serde(default, skip_serializing_if = "Option::is_none")]
pub agent_id: Option<String>,
#[serde(default, skip_serializing_if = "Option::is_none")]
pub agent_type: Option<String>,
```

Every `Request` struct must carry these. The core `hook_runtime.rs` must populate them from the current agent context (if any). Start in Phase 1 — every CommandInput struct from that point forward includes them.

### 2. `allowManagedHooksOnly` policy setting (Phase 3)

Enterprise administrators can set `"allowManagedHooksOnly": true` in managed policy settings. When set, **all** user, project, local, and plugin hooks are blocked — only managed hooks execute.

**Implementation**: In `discovery.rs`, after scanning all layers, if the managed layer has `allowManagedHooksOnly: true`, filter the handler list to only those whose `source_path` originates from the managed layer.

Add to Phase 3 alongside `disableAllHooks`.

### 3. Plugin hooks discovery (Phase 3)

Plugins can define hooks in `hooks/hooks.json` within their installation directory. These hooks merge with user and project hooks when the plugin is enabled.

**Implementation**: If plugins are already represented as a config layer in `ConfigLayerStack`, this works automatically. If not, `discovery.rs` must also scan enabled plugin directories for `hooks/hooks.json`.

The following env vars must be set on child processes for plugin hooks:
- `CLAUDE_PLUGIN_ROOT` — the plugin's installation directory
- `CLAUDE_PLUGIN_DATA` — the plugin's persistent data directory

Add to Phase 3 alongside `CLAUDE_PROJECT_DIR`.

### 4. Permission update entries — typed parsing in core (Phase 1)

Phase 1 passes `updatedPermissions` as `Option<Vec<serde_json::Value>>` through the hooks crate. This is correct — the hooks crate is transport-agnostic.

However, **the core crate must parse these into typed structures** to actually apply them:

```rust
// In codex-core, not codex-hooks:
pub enum PermissionUpdateEntry {
    AddRules { rules: Vec<PermissionRule>, behavior: RuleBehavior, destination: SettingsDestination },
    ReplaceRules { rules: Vec<PermissionRule>, behavior: RuleBehavior, destination: SettingsDestination },
    RemoveRules { rules: Vec<PermissionRule>, behavior: RuleBehavior, destination: SettingsDestination },
    SetMode { mode: String, destination: SettingsDestination },
    AddDirectories { directories: Vec<PathBuf>, destination: SettingsDestination },
    RemoveDirectories { directories: Vec<PathBuf>, destination: SettingsDestination },
}

pub struct PermissionRule {
    pub tool_name: String,
    pub rule_content: Option<String>,
}

pub enum SettingsDestination {
    Session,
    LocalSettings,
    ProjectSettings,
    UserSettings,
}
```

This parsing and application logic lives in `core/src/hook_runtime.rs` as part of the `PermissionRequestHookDisposition::Allow` handler.

### 5. Hook source labels on "ask" prompts (Phase 1)

When a PreToolUse hook returns `permissionDecision: "ask"`, the permission prompt shown to the user includes a label identifying the hook's origin: `[User]`, `[Project]`, `[Plugin]`, `[Local]`.

**Implementation**: `ConfiguredHandler` already has `source_path`. Derive the source label from which config layer the handler came from. Pass the label through `PreToolUseHookDisposition::Ask` so the TUI can display it.

### 6. `CLAUDE_CODE_REMOTE` environment variable (Phase 3)

Per spec, the `$CLAUDE_CODE_REMOTE` env var is set to `"true"` in remote web environments and not set in the local CLI. Set this on hook child processes in `command_runner.rs` based on the runtime context.
