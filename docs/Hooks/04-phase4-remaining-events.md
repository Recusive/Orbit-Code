# Phase 4 — Remaining Events & Prompt/Agent Hooks

## Scope

Eight remaining events and two handler types:

| Component | Type | Description |
|-----------|------|-------------|
| **PreCompact** | Event | Before context compaction |
| **PostCompact** | Event | After context compaction |
| **WorktreeCreate** | Event | When worktree is being created |
| **WorktreeRemove** | Event | When worktree is being removed |
| **TeammateIdle** | Event | When agent team teammate goes idle |
| **TaskCompleted** | Event | When task is marked completed |
| **Elicitation** | Event | When MCP server requests user input |
| **ElicitationResult** | Event | After user responds to elicitation |
| **Prompt hooks** | Handler type | Single-turn LLM evaluation |
| **Agent hooks** | Handler type | Multi-turn subagent verification |

## Dependencies

Phases 1-3 must be complete. All infrastructure (HTTP runner, async runner, dispatcher, discovery patterns) is established.

---

## 1. PreCompact Event

### 1.1 Schema

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "pre-compact.command.input")]
pub(crate) struct PreCompactCommandInput {
    pub session_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "pre_compact_hook_event_name_schema")]
    pub hook_event_name: String,
    #[schemars(schema_with = "compact_trigger_schema")]
    pub trigger: String,
    #[serde(default)]
    pub custom_instructions: Option<String>,
}
```

Trigger values: `manual` (`/compact`), `auto` (context window full).

Output: No decision control. Universal fields only.

### 1.2 Event Module

```rust
#[derive(Debug, Clone)]
pub struct PreCompactRequest {
    pub session_id: ThreadId,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub trigger: CompactTrigger,
    pub custom_instructions: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum CompactTrigger {
    Manual,
    Auto,
}

impl CompactTrigger {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Auto => "auto",
        }
    }
}

#[derive(Debug)]
pub struct PreCompactOutcome {
    pub hook_events: Vec<HookCompletedEvent>,
    // No decision control. Side effects only.
}
```

Matcher matches on `trigger.as_str()`. Exit code 2 = non-blocking (stderr shown to user only).

### 1.3 Core Integration

Wire into `codex-rs/core/src/compact.rs` or wherever compaction is triggered. Fire before compaction starts.

---

## 2. PostCompact Event

### 2.1 Schema

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "post-compact.command.input")]
pub(crate) struct PostCompactCommandInput {
    pub session_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "post_compact_hook_event_name_schema")]
    pub hook_event_name: String,
    #[schemars(schema_with = "compact_trigger_schema")]
    pub trigger: String,
    pub compact_summary: String,
}
```

### 2.2 Event Module

```rust
#[derive(Debug, Clone)]
pub struct PostCompactRequest {
    pub session_id: ThreadId,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub trigger: CompactTrigger,
    pub compact_summary: String,
}

#[derive(Debug)]
pub struct PostCompactOutcome {
    pub hook_events: Vec<HookCompletedEvent>,
    // No decision control.
}
```

Reuse `CompactTrigger` from pre_compact. Same matcher behavior.

### 2.3 Core Integration

Fire after compaction completes, passing the generated summary.

---

## 3. WorktreeCreate Event

### 3.1 Schema

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "worktree-create.command.input")]
pub(crate) struct WorktreeCreateCommandInput {
    pub session_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "worktree_create_hook_event_name_schema")]
    pub hook_event_name: String,
    pub name: String,
}
```

Output: Hook prints the **absolute path** to the created worktree on stdout. No JSON output schema — raw stdout path.

### 3.2 Event Module

```rust
#[derive(Debug, Clone)]
pub struct WorktreeCreateRequest {
    pub session_id: ThreadId,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub name: String,
}

#[derive(Debug)]
pub struct WorktreeCreateOutcome {
    pub hook_events: Vec<HookCompletedEvent>,
    pub worktree_path: Option<PathBuf>,
    pub failed: bool,
    pub error_message: Option<String>,
}
```

**Special behavior**: This hook replaces the default git worktree behavior. The hook's stdout is the worktree path. Non-zero exit = creation failure. Only `type: "command"` hooks are supported.

No matcher support (always fires on every occurrence).

### 3.3 Run Logic

```rust
fn parse_completed(handler, run_result, turn_id) -> ParsedHandler<WorktreeCreateData> {
    match run_result.exit_code {
        Some(0) => {
            let path = run_result.stdout.trim();
            if path.is_empty() {
                // No output = failure
                WorktreeCreateData { worktree_path: None, failed: true, error: Some("...") }
            } else {
                WorktreeCreateData {
                    worktree_path: Some(PathBuf::from(path)),
                    failed: false,
                    error: None,
                }
            }
        }
        _ => {
            // Any non-zero exit = failure
            WorktreeCreateData { worktree_path: None, failed: true, error: Some("...") }
        }
    }
}
```

### 3.4 Core Integration

Wire into worktree creation code. If a WorktreeCreate hook is configured, it replaces the default `git worktree add` behavior.

---

## 4. WorktreeRemove Event

### 4.1 Schema

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "worktree-remove.command.input")]
pub(crate) struct WorktreeRemoveCommandInput {
    pub session_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "worktree_remove_hook_event_name_schema")]
    pub hook_event_name: String,
    pub worktree_path: String,
}
```

### 4.2 Event Module

```rust
#[derive(Debug, Clone)]
pub struct WorktreeRemoveRequest {
    pub session_id: ThreadId,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub worktree_path: PathBuf,
}

#[derive(Debug)]
pub struct WorktreeRemoveOutcome {
    pub hook_events: Vec<HookCompletedEvent>,
    // No decision control. Failures logged in debug mode only.
}
```

No decision control. No matcher support. Failures are logged in debug mode only. Only `type: "command"` hooks supported.

---

## 5. TeammateIdle Event

### 5.1 Schema

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "teammate-idle.command.input")]
pub(crate) struct TeammateIdleCommandInput {
    pub session_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "teammate_idle_hook_event_name_schema")]
    pub hook_event_name: String,
    #[schemars(schema_with = "permission_mode_schema")]
    pub permission_mode: String,
    pub teammate_name: String,
    pub team_name: String,
}
```

Output: Exit code 2 blocks (teammate continues). JSON `continue:false` stops teammate entirely.

### 5.2 Event Module

```rust
#[derive(Debug, Clone)]
pub struct TeammateIdleRequest {
    pub session_id: ThreadId,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub permission_mode: String,
    pub teammate_name: String,
    pub team_name: String,
}

#[derive(Debug)]
pub struct TeammateIdleOutcome {
    pub hook_events: Vec<HookCompletedEvent>,
    pub should_stop: bool,
    pub stop_reason: Option<String>,
    pub should_continue_working: bool,
    pub continuation_feedback: Option<String>,
}
```

No matcher support (always fires).

**Decision control**:
- Exit code 2: `should_continue_working = true`, stderr → `continuation_feedback`
- JSON `continue:false`: `should_stop = true`, stopReason → `stop_reason`
- Exit 0 with no JSON: `should_continue_working = false`, teammate goes idle

---

## 6. TaskCompleted Event

### 6.1 Schema

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "task-completed.command.input")]
pub(crate) struct TaskCompletedCommandInput {
    pub session_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "task_completed_hook_event_name_schema")]
    pub hook_event_name: String,
    #[schemars(schema_with = "permission_mode_schema")]
    pub permission_mode: String,
    pub task_id: String,
    pub task_subject: String,
    #[serde(default)]
    pub task_description: Option<String>,
    #[serde(default)]
    pub teammate_name: Option<String>,
    #[serde(default)]
    pub team_name: Option<String>,
}
```

### 6.2 Event Module

```rust
#[derive(Debug, Clone)]
pub struct TaskCompletedRequest {
    pub session_id: ThreadId,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub permission_mode: String,
    pub task_id: String,
    pub task_subject: String,
    pub task_description: Option<String>,
    pub teammate_name: Option<String>,
    pub team_name: Option<String>,
}

#[derive(Debug)]
pub struct TaskCompletedOutcome {
    pub hook_events: Vec<HookCompletedEvent>,
    pub should_stop: bool,
    pub stop_reason: Option<String>,
    pub should_block_completion: bool,
    pub block_feedback: Option<String>,
}
```

No matcher support. Same decision control as TeammateIdle:
- Exit code 2: `should_block_completion = true`, stderr → `block_feedback` (task not marked complete)
- JSON `continue:false`: `should_stop = true` (stops teammate entirely)

---

## 7. Elicitation Event

### 7.1 Schema

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "elicitation.command.input")]
pub(crate) struct ElicitationCommandInput {
    pub session_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "elicitation_hook_event_name_schema")]
    pub hook_event_name: String,
    #[schemars(schema_with = "permission_mode_schema")]
    pub permission_mode: String,
    pub mcp_server_name: String,
    pub message: String,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub elicitation_id: Option<String>,
    #[serde(default)]
    pub requested_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[schemars(rename = "elicitation.command.output")]
pub(crate) struct ElicitationCommandOutputWire {
    #[serde(flatten)]
    pub universal: HookUniversalOutputWire,
    #[serde(default)]
    pub hook_specific_output: Option<ElicitationHookSpecificOutputWire>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub(crate) struct ElicitationHookSpecificOutputWire {
    pub hook_event_name: HookEventNameWire,
    #[serde(default)]
    pub action: Option<ElicitationActionWire>,
    #[serde(default)]
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub(crate) enum ElicitationActionWire {
    #[serde(rename = "accept")]
    Accept,
    #[serde(rename = "decline")]
    Decline,
    #[serde(rename = "cancel")]
    Cancel,
}
```

Matcher matches on `mcp_server_name`.

### 7.2 Event Module

```rust
#[derive(Debug, Clone)]
pub struct ElicitationRequest {
    pub session_id: ThreadId,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub permission_mode: String,
    pub mcp_server_name: String,
    pub message: String,
    pub mode: Option<String>,
    pub url: Option<String>,
    pub elicitation_id: Option<String>,
    pub requested_schema: Option<serde_json::Value>,
}

#[derive(Debug)]
pub struct ElicitationOutcome {
    pub hook_events: Vec<HookCompletedEvent>,
    pub should_stop: bool,
    pub stop_reason: Option<String>,
    pub action: Option<ElicitationAction>,
    pub content: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElicitationAction {
    Accept,
    Decline,
    Cancel,
}
```

Exit code 2: denies the elicitation.

Only `type: "command"` hooks supported.

---

## 8. ElicitationResult Event

### 8.1 Schema

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "elicitation-result.command.input")]
pub(crate) struct ElicitationResultCommandInput {
    pub session_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "elicitation_result_hook_event_name_schema")]
    pub hook_event_name: String,
    #[schemars(schema_with = "permission_mode_schema")]
    pub permission_mode: String,
    pub mcp_server_name: String,
    #[schemars(schema_with = "elicitation_action_schema")]
    pub action: String,
    #[serde(default)]
    pub content: Option<serde_json::Value>,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub elicitation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[schemars(rename = "elicitation-result.command.output")]
pub(crate) struct ElicitationResultCommandOutputWire {
    #[serde(flatten)]
    pub universal: HookUniversalOutputWire,
    #[serde(default)]
    pub hook_specific_output: Option<ElicitationResultHookSpecificOutputWire>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub(crate) struct ElicitationResultHookSpecificOutputWire {
    pub hook_event_name: HookEventNameWire,
    #[serde(default)]
    pub action: Option<ElicitationActionWire>,
    #[serde(default)]
    pub content: Option<serde_json::Value>,
}
```

### 8.2 Event Module

Same structure as Elicitation. Hook can override user's action and content. Exit code 2 blocks the response (action becomes decline). Only `type: "command"` hooks supported.

---

## 9. Prompt Hooks (`type: "prompt"`)

### 9.1 Config

Already stubbed in `HookHandlerConfig::Prompt {}`. Extend:

```rust
#[serde(rename = "prompt")]
Prompt {
    prompt: String,
    #[serde(default)]
    model: Option<String>,
    #[serde(default, rename = "timeout", alias = "timeoutSec")]
    timeout_sec: Option<u64>,
    #[serde(default, rename = "statusMessage")]
    status_message: Option<String>,
},
```

### 9.2 ConfiguredHandler

Extend `HandlerKind`:

```rust
pub(crate) enum HandlerKind {
    Command { command: String, is_async: bool },
    Http { url: String, headers: HashMap<String, String>, allowed_env_vars: Vec<String> },
    Prompt { prompt: String, model: Option<String> },
    Agent { prompt: String, model: Option<String> },
}
```

### 9.3 Prompt Runner

#### New file: `codex-rs/hooks/src/engine/prompt_runner.rs`

```rust
use super::ConfiguredHandler;
use super::command_runner::CommandRunResult;

/// Run a prompt hook by calling a Claude model with the prompt.
///
/// The model receives the prompt (with $ARGUMENTS replaced by input JSON)
/// and must return JSON: {"ok": true} or {"ok": false, "reason": "..."}
///
/// This is mapped to CommandRunResult for compatibility:
/// - ok: true → exit 0, empty stdout
/// - ok: false → exit 0, stdout with {"decision":"block","reason":"..."}
pub(crate) async fn run_prompt(
    handler: &ConfiguredHandler,
    input_json: &str,
    model_client: &dyn PromptHookModelClient,
) -> CommandRunResult {
    let HandlerKind::Prompt { prompt, model } = &handler.kind else {
        return error_result("handler is not a prompt hook");
    };

    // Replace $ARGUMENTS with input JSON
    let full_prompt = if prompt.contains("$ARGUMENTS") {
        prompt.replace("$ARGUMENTS", input_json)
    } else {
        format!("{prompt}\n\n{input_json}")
    };

    let model_id = model.as_deref().unwrap_or("haiku"); // default to fast model

    match model_client.evaluate_prompt(&full_prompt, model_id).await {
        Ok(response) => parse_prompt_response(&response),
        Err(err) => error_result(&format!("prompt hook failed: {err}")),
    }
}

fn parse_prompt_response(response: &str) -> CommandRunResult {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(response) else {
        return error_result("prompt hook returned non-JSON response");
    };
    let ok = value.get("ok").and_then(|v| v.as_bool()).unwrap_or(true);
    if ok {
        CommandRunResult {
            started_at: chrono::Utc::now().timestamp(),
            completed_at: chrono::Utc::now().timestamp(),
            duration_ms: 0,
            exit_code: Some(0),
            stdout: String::new(),
            stderr: String::new(),
            error: None,
        }
    } else {
        let reason = value
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("Prompt hook rejected the action")
            .to_string();
        // Map to decision:block for compatibility with existing parse pipeline
        let output = serde_json::json!({
            "decision": "block",
            "reason": reason,
        });
        CommandRunResult {
            started_at: chrono::Utc::now().timestamp(),
            completed_at: chrono::Utc::now().timestamp(),
            duration_ms: 0,
            exit_code: Some(0),
            stdout: output.to_string(),
            stderr: String::new(),
            error: None,
        }
    }
}

/// Trait for LLM evaluation — allows mocking in tests.
#[async_trait::async_trait]
pub(crate) trait PromptHookModelClient: Send + Sync {
    async fn evaluate_prompt(
        &self,
        prompt: &str,
        model: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}
```

### 9.4 Supported Events

Prompt hooks support these events only:
- PermissionRequest
- PostToolUse
- PostToolUseFailure
- PreToolUse
- Stop
- SubagentStop
- TaskCompleted
- UserPromptSubmit

All other events reject prompt hooks during discovery with a warning.

### 9.5 Tests

- `prompt_hook_ok_true_allows`
- `prompt_hook_ok_false_blocks_with_reason`
- `prompt_hook_arguments_placeholder_replaced`
- `prompt_hook_default_model_is_haiku`
- `prompt_hook_non_json_response_fails`
- `prompt_hook_timeout`

---

## 10. Agent Hooks (`type: "agent"`)

### 10.1 Agent Runner

#### New file: `codex-rs/hooks/src/engine/agent_runner.rs`

Agent hooks spawn a subagent with read-only tools (Read, Grep, Glob). The subagent evaluates the prompt and returns `{ok: true/false}`.

```rust
/// Run an agent hook by spawning a subagent with tool access.
///
/// The subagent can use Read, Grep, Glob to investigate.
/// After up to 50 turns, it returns {"ok": true} or {"ok": false, "reason": "..."}.
///
/// Response is mapped to CommandRunResult for pipeline compatibility.
pub(crate) async fn run_agent(
    handler: &ConfiguredHandler,
    input_json: &str,
    agent_spawner: &dyn AgentHookSpawner,
) -> CommandRunResult {
    let HandlerKind::Agent { prompt, model } = &handler.kind else {
        return error_result("handler is not an agent hook");
    };

    let full_prompt = if prompt.contains("$ARGUMENTS") {
        prompt.replace("$ARGUMENTS", input_json)
    } else {
        format!("{prompt}\n\n{input_json}")
    };

    let model_id = model.as_deref().unwrap_or("haiku");

    match agent_spawner
        .spawn_verification_agent(&full_prompt, model_id, /*max_turns*/ 50)
        .await
    {
        Ok(response) => parse_prompt_response(&response), // Same format as prompt hooks
        Err(err) => error_result(&format!("agent hook failed: {err}")),
    }
}

#[async_trait::async_trait]
pub(crate) trait AgentHookSpawner: Send + Sync {
    async fn spawn_verification_agent(
        &self,
        prompt: &str,
        model: &str,
        max_turns: u32,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}
```

### 10.2 Supported Events

Same as prompt hooks. Agent hooks support the same 8 events. Default timeout: 60s (vs 30s for prompt hooks).

### 10.3 Tests

- `agent_hook_ok_true_allows`
- `agent_hook_ok_false_blocks_with_reason`
- `agent_hook_timeout`
- `agent_hook_max_turns_reached`

---

## 11. Dispatcher Update

### File: `codex-rs/hooks/src/engine/dispatcher.rs`

`execute_handlers` routes all four handler types:

```rust
match &handler.kind {
    HandlerKind::Command { is_async: false, .. } => {
        command_runner::run_command(shell, handler, &input_json, cwd).await
    }
    HandlerKind::Command { is_async: true, .. } => {
        // Handled separately before this point (spawned async)
        unreachable!("async handlers filtered out before execute_handlers")
    }
    HandlerKind::Http { .. } => {
        http_runner::run_http(handler, &input_json, cwd).await
    }
    HandlerKind::Prompt { .. } => {
        prompt_runner::run_prompt(handler, &input_json, model_client).await
    }
    HandlerKind::Agent { .. } => {
        agent_runner::run_agent(handler, &input_json, agent_spawner).await
    }
}
```

This means `execute_handlers` needs `model_client` and `agent_spawner` parameters. These come from the `HooksConfig` (passed through from core):

```rust
pub struct HooksConfig {
    pub legacy_notify_argv: Option<Vec<String>>,
    pub feature_enabled: bool,
    pub config_layer_stack: Option<ConfigLayerStack>,
    pub shell_program: Option<String>,
    pub shell_args: Vec<String>,
    // Phase 4 additions:
    pub model_client: Option<Arc<dyn PromptHookModelClient>>,
    pub agent_spawner: Option<Arc<dyn AgentHookSpawner>>,
}
```

If `model_client` is None and a prompt hook is discovered, it becomes a startup warning.

---

## 12. `once` Field (Skills/Agents Only)

Per spec: hooks defined in skill/agent frontmatter can set `once: true` to run only once per session, then be removed.

### Implementation

Add `once: bool` to `ConfiguredHandler`. After a `once` handler executes, mark it for removal. The engine maintains a `HashSet<String>` of completed once-handler IDs:

```rust
pub(crate) struct ClaudeHooksEngine {
    handlers: Vec<ConfiguredHandler>,
    completed_once_handlers: std::sync::Mutex<HashSet<String>>,
    // ...
}
```

In `select_handlers`, filter out handlers that are in `completed_once_handlers`. After execution, add `once` handlers to the set.

---

## 13. Hooks in Skills/Agents Frontmatter

Per spec: hooks can be defined in YAML frontmatter of skill and agent files. These hooks are scoped to the component's lifecycle.

### Implementation

When a skill or agent is loaded, parse its frontmatter for hook definitions. Register these as `ConfiguredHandler` entries with a `SessionScoped` source type. When the skill/agent deactivates, remove the handlers.

For subagents, `Stop` hooks in frontmatter are automatically converted to `SubagentStop` (per spec).

This requires:
1. A method on the engine to add/remove handlers at runtime: `register_session_hooks()`, `unregister_session_hooks()`
2. The discovery result to be mutable (currently static at init)

---

## 14. /hooks Menu

Per spec: `/hooks` opens a read-only browser showing all configured hooks. This is a TUI feature.

### Implementation

Add a `/hooks` command handler in the TUI that:
1. Calls `sess.hooks().list_all_handlers()` (new method)
2. Displays event names with hook counts
3. Allows drilling into matchers
4. Shows full handler details (type, source, command/prompt/URL)
5. Labels sources: User, Project, Local, Plugin, Session, Built-in

### File: `codex-rs/tui/src/` (new hooks browser widget)

Implement as a scrollable list with detail view, similar to existing TUI menus.

---

## 15. Protocol — Final HookEventName

After all 4 phases, the complete `HookEventName` enum:

```rust
pub enum HookEventName {
    SessionStart,
    UserPromptSubmit,
    PreToolUse,
    PermissionRequest,
    PostToolUse,
    PostToolUseFailure,
    Notification,
    SubagentStart,
    SubagentStop,
    Stop,
    StopFailure,
    TeammateIdle,
    TaskCompleted,
    InstructionsLoaded,
    ConfigChange,
    WorktreeCreate,
    WorktreeRemove,
    PreCompact,
    PostCompact,
    Elicitation,
    ElicitationResult,
    SessionEnd,
}
```

21 variants. Exact parity with Claude Code.

---

## 16. Schema Fixtures — Final Count

After all 4 phases:

| Phase | Events | Fixtures |
|-------|--------|----------|
| Existing | 3 | 6 |
| Phase 1 | 4 | 8 |
| Phase 2 | 5 | 10 |
| Phase 3 | 2 | 4 |
| Phase 4 | 8 | 16 |
| **Total** | **22** | **44** |

(22 = 21 events + the 1 existing SessionStart which has input + output = 2 files, etc.)

---

## 17. Checklist

- [ ] PreCompact event: schema, event module, core integration
- [ ] PostCompact event: schema, event module, core integration
- [ ] WorktreeCreate event: schema, event module, core integration
- [ ] WorktreeRemove event: schema, event module, core integration
- [ ] TeammateIdle event: schema, event module, core integration
- [ ] TaskCompleted event: schema, event module, core integration
- [ ] Elicitation event: schema, event module, core integration
- [ ] ElicitationResult event: schema, event module, core integration
- [ ] `HookHandlerConfig::Prompt` fully implemented with prompt, model, timeout
- [ ] `HookHandlerConfig::Agent` fully implemented with prompt, model, timeout
- [ ] `HandlerKind::Prompt` and `HandlerKind::Agent` added
- [ ] `prompt_runner.rs` created with LLM evaluation logic
- [ ] `agent_runner.rs` created with subagent spawning logic
- [ ] `PromptHookModelClient` trait defined and implemented
- [ ] `AgentHookSpawner` trait defined and implemented
- [ ] Prompt/agent hooks restricted to supported events (8 events)
- [ ] `$ARGUMENTS` placeholder replaced in prompt text
- [ ] `{ok: true/false, reason: "..."}` response schema enforced
- [ ] Default model for prompt/agent hooks (haiku)
- [ ] Default timeout: 30s prompt, 60s agent
- [ ] `once` field implemented for skill/agent frontmatter hooks
- [ ] Skill/agent frontmatter hook registration/deregistration
- [ ] Subagent `Stop` → `SubagentStop` conversion for agent frontmatter
- [ ] `/hooks` TUI menu implemented
- [ ] `HooksConfig` extended with model_client and agent_spawner
- [ ] Dispatcher routes to prompt_runner and agent_runner
- [ ] All 16 new schema fixtures generated and committed
- [ ] All event modules have comprehensive tests
- [ ] All core integration points wired
- [ ] TUI snapshot tests for new events
- [ ] `just fmt`, `just fix`, `cargo test` all pass
- [ ] `HookEventName` has exactly 21 variants
- [ ] Total schema fixtures: 44 files
- [ ] Feature complete: exact parity with Claude Code hooks system
