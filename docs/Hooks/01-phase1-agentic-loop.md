# Phase 1 — Core Agentic Loop Hooks

## Scope

Four events that fire inside the tool execution cycle:

| Event | When | Matcher | Can Block? |
|-------|------|---------|-----------|
| **PreToolUse** | After Claude generates tool params, before execution | tool name | Yes (allow/deny/ask) |
| **PostToolUse** | After a tool completes successfully | tool name | Yes (decision:block) |
| **PostToolUseFailure** | After a tool fails | tool name | No (context only) |
| **PermissionRequest** | When a permission dialog is about to show | tool name | Yes (allow/deny) |

These are the highest-impact hooks — they control whether tools run, modify tool inputs, and provide feedback after execution.

---

## 1. Protocol Changes

### File: `codex-rs/protocol/src/protocol.rs`

#### 1.1 Extend `HookEventName`

```rust
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
pub enum HookEventName {
    SessionStart,
    UserPromptSubmit,
    Stop,
    // Phase 1 additions:
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    PermissionRequest,
}
```

#### 1.2 Update `HookScope`

Add a `ToolCall` scope for tool-level hooks (scoped to a single tool invocation within a turn):

```rust
pub enum HookScope {
    Thread,
    Turn,
    ToolCall, // new
}
```

No existing consumers break — the scope is informational metadata on `HookRunSummary`.

---

## 2. Hooks Crate — Schema Wire Types

### File: `codex-rs/hooks/src/schema.rs`

#### 2.1 New `HookEventNameWire` variants

```rust
pub(crate) enum HookEventNameWire {
    SessionStart,
    UserPromptSubmit,
    Stop,
    #[serde(rename = "PreToolUse")]
    PreToolUse,
    #[serde(rename = "PostToolUse")]
    PostToolUse,
    #[serde(rename = "PostToolUseFailure")]
    PostToolUseFailure,
    #[serde(rename = "PermissionRequest")]
    PermissionRequest,
}
```

#### 2.2 PreToolUse Input Schema

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "pre-tool-use.command.input")]
pub(crate) struct PreToolUseCommandInput {
    pub session_id: String,
    pub turn_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "pre_tool_use_hook_event_name_schema")]
    pub hook_event_name: String,
    #[schemars(schema_with = "permission_mode_schema")]
    pub permission_mode: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub tool_use_id: String,
}
```

#### 2.3 PreToolUse Output Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[schemars(rename = "pre-tool-use.command.output")]
pub(crate) struct PreToolUseCommandOutputWire {
    #[serde(flatten)]
    pub universal: HookUniversalOutputWire,
    #[serde(default)]
    pub hook_specific_output: Option<PreToolUseHookSpecificOutputWire>,
    // Deprecated top-level fields (backward compat):
    #[serde(default)]
    pub decision: Option<PreToolUseDeprecatedDecisionWire>,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub(crate) struct PreToolUseHookSpecificOutputWire {
    pub hook_event_name: HookEventNameWire,
    #[serde(default)]
    pub permission_decision: Option<PreToolUsePermissionDecisionWire>,
    #[serde(default)]
    pub permission_decision_reason: Option<String>,
    #[serde(default)]
    pub updated_input: Option<serde_json::Value>,
    #[serde(default)]
    pub additional_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub(crate) enum PreToolUsePermissionDecisionWire {
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "deny")]
    Deny,
    #[serde(rename = "ask")]
    Ask,
}

/// Backward-compatible deprecated decision values.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub(crate) enum PreToolUseDeprecatedDecisionWire {
    #[serde(rename = "approve")]
    Approve,
    #[serde(rename = "block")]
    Block,
}
```

#### 2.4 PostToolUse Input/Output Schemas

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "post-tool-use.command.input")]
pub(crate) struct PostToolUseCommandInput {
    pub session_id: String,
    pub turn_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "post_tool_use_hook_event_name_schema")]
    pub hook_event_name: String,
    #[schemars(schema_with = "permission_mode_schema")]
    pub permission_mode: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub tool_response: serde_json::Value,
    pub tool_use_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[schemars(rename = "post-tool-use.command.output")]
pub(crate) struct PostToolUseCommandOutputWire {
    #[serde(flatten)]
    pub universal: HookUniversalOutputWire,
    #[serde(default)]
    pub decision: Option<BlockDecisionWire>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub hook_specific_output: Option<PostToolUseHookSpecificOutputWire>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub(crate) struct PostToolUseHookSpecificOutputWire {
    pub hook_event_name: HookEventNameWire,
    #[serde(default)]
    pub additional_context: Option<String>,
    #[serde(default)]
    pub updated_mcp_tool_output: Option<serde_json::Value>,
}
```

#### 2.5 PostToolUseFailure Input/Output Schemas

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "post-tool-use-failure.command.input")]
pub(crate) struct PostToolUseFailureCommandInput {
    pub session_id: String,
    pub turn_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "post_tool_use_failure_hook_event_name_schema")]
    pub hook_event_name: String,
    #[schemars(schema_with = "permission_mode_schema")]
    pub permission_mode: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub tool_use_id: String,
    pub error: String,
    #[serde(default)]
    pub is_interrupt: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[schemars(rename = "post-tool-use-failure.command.output")]
pub(crate) struct PostToolUseFailureCommandOutputWire {
    #[serde(flatten)]
    pub universal: HookUniversalOutputWire,
    #[serde(default)]
    pub hook_specific_output: Option<PostToolUseFailureHookSpecificOutputWire>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub(crate) struct PostToolUseFailureHookSpecificOutputWire {
    pub hook_event_name: HookEventNameWire,
    #[serde(default)]
    pub additional_context: Option<String>,
}
```

#### 2.6 PermissionRequest Input/Output Schemas

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "permission-request.command.input")]
pub(crate) struct PermissionRequestCommandInput {
    pub session_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "permission_request_hook_event_name_schema")]
    pub hook_event_name: String,
    #[schemars(schema_with = "permission_mode_schema")]
    pub permission_mode: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    #[serde(default)]
    pub permission_suggestions: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[schemars(rename = "permission-request.command.output")]
pub(crate) struct PermissionRequestCommandOutputWire {
    #[serde(flatten)]
    pub universal: HookUniversalOutputWire,
    #[serde(default)]
    pub hook_specific_output: Option<PermissionRequestHookSpecificOutputWire>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub(crate) struct PermissionRequestHookSpecificOutputWire {
    pub hook_event_name: HookEventNameWire,
    #[serde(default)]
    pub decision: Option<PermissionRequestDecisionWire>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub(crate) struct PermissionRequestDecisionWire {
    pub behavior: PermissionRequestBehaviorWire,
    #[serde(default)]
    pub updated_input: Option<serde_json::Value>,
    #[serde(default)]
    pub updated_permissions: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub interrupt: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub(crate) enum PermissionRequestBehaviorWire {
    #[serde(rename = "allow")]
    Allow,
    #[serde(rename = "deny")]
    Deny,
}
```

#### 2.7 Schema Fixtures

Add to `write_schema_fixtures()`:

```rust
const PRE_TOOL_USE_INPUT_FIXTURE: &str = "pre-tool-use.command.input.schema.json";
const PRE_TOOL_USE_OUTPUT_FIXTURE: &str = "pre-tool-use.command.output.schema.json";
const POST_TOOL_USE_INPUT_FIXTURE: &str = "post-tool-use.command.input.schema.json";
const POST_TOOL_USE_OUTPUT_FIXTURE: &str = "post-tool-use.command.output.schema.json";
const POST_TOOL_USE_FAILURE_INPUT_FIXTURE: &str = "post-tool-use-failure.command.input.schema.json";
const POST_TOOL_USE_FAILURE_OUTPUT_FIXTURE: &str = "post-tool-use-failure.command.output.schema.json";
const PERMISSION_REQUEST_INPUT_FIXTURE: &str = "permission-request.command.input.schema.json";
const PERMISSION_REQUEST_OUTPUT_FIXTURE: &str = "permission-request.command.output.schema.json";
```

---

## 3. Hooks Crate — Engine Config

### File: `codex-rs/hooks/src/engine/config.rs`

```rust
#[derive(Debug, Default, Deserialize)]
pub(crate) struct HookEvents {
    #[serde(rename = "SessionStart", default)]
    pub session_start: Vec<MatcherGroup>,
    #[serde(rename = "UserPromptSubmit", default)]
    pub user_prompt_submit: Vec<MatcherGroup>,
    #[serde(rename = "Stop", default)]
    pub stop: Vec<MatcherGroup>,
    // Phase 1:
    #[serde(rename = "PreToolUse", default)]
    pub pre_tool_use: Vec<MatcherGroup>,
    #[serde(rename = "PostToolUse", default)]
    pub post_tool_use: Vec<MatcherGroup>,
    #[serde(rename = "PostToolUseFailure", default)]
    pub post_tool_use_failure: Vec<MatcherGroup>,
    #[serde(rename = "PermissionRequest", default)]
    pub permission_request: Vec<MatcherGroup>,
}
```

---

## 4. Hooks Crate — Discovery

### File: `codex-rs/hooks/src/engine/discovery.rs`

Add 4 new discovery loops inside `discover_handlers()`, after the existing `stop` loop:

```rust
for group in parsed.hooks.pre_tool_use {
    append_group_handlers(
        &mut handlers, &mut warnings, &mut display_order,
        source_path.as_path(),
        HookEventName::PreToolUse,
        effective_matcher(HookEventName::PreToolUse, group.matcher.as_deref()),
        group.hooks,
    );
}
// Same pattern for PostToolUse, PostToolUseFailure, PermissionRequest
```

Update `effective_matcher()` — all four Phase 1 events support matchers (tool name):

```rust
fn effective_matcher(event_name: HookEventName, matcher: Option<&str>) -> Option<&str> {
    match event_name {
        HookEventName::SessionStart
        | HookEventName::PreToolUse
        | HookEventName::PostToolUse
        | HookEventName::PostToolUseFailure
        | HookEventName::PermissionRequest => matcher,
        HookEventName::UserPromptSubmit | HookEventName::Stop => None,
    }
}
```

---

## 5. Hooks Crate — Dispatcher

### File: `codex-rs/hooks/src/engine/dispatcher.rs`

#### 5.1 Update `select_handlers`

Tool-name matching uses the same regex pattern as SessionStart:

```rust
pub(crate) fn select_handlers(
    handlers: &[ConfiguredHandler],
    event_name: HookEventName,
    matcher_input: Option<&str>,
) -> Vec<ConfiguredHandler> {
    handlers
        .iter()
        .filter(|handler| handler.event_name == event_name)
        .filter(|handler| match event_name {
            // Events that support regex matchers:
            HookEventName::SessionStart
            | HookEventName::PreToolUse
            | HookEventName::PostToolUse
            | HookEventName::PostToolUseFailure
            | HookEventName::PermissionRequest => {
                match (&handler.matcher, matcher_input) {
                    (Some(matcher), Some(input)) => regex::Regex::new(matcher)
                        .map(|regex| regex.is_match(input))
                        .unwrap_or(false),
                    (None, _) => true,
                    _ => false,
                }
            }
            // Events with no matcher support:
            HookEventName::UserPromptSubmit | HookEventName::Stop => true,
        })
        .cloned()
        .collect()
}
```

#### 5.2 Update `scope_for_event`

```rust
fn scope_for_event(event_name: HookEventName) -> HookScope {
    match event_name {
        HookEventName::SessionStart => HookScope::Thread,
        HookEventName::UserPromptSubmit | HookEventName::Stop => HookScope::Turn,
        HookEventName::PreToolUse
        | HookEventName::PostToolUse
        | HookEventName::PostToolUseFailure
        | HookEventName::PermissionRequest => HookScope::ToolCall,
    }
}
```

---

## 6. Hooks Crate — Output Parser

### File: `codex-rs/hooks/src/engine/output_parser.rs`

#### 6.1 New Output Structs

```rust
#[derive(Debug, Clone)]
pub(crate) struct PreToolUseOutput {
    pub universal: UniversalOutput,
    pub permission_decision: Option<PreToolUsePermissionDecision>,
    pub permission_decision_reason: Option<String>,
    pub updated_input: Option<serde_json::Value>,
    pub additional_context: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PreToolUsePermissionDecision {
    Allow,
    Deny,
    Ask,
}

#[derive(Debug, Clone)]
pub(crate) struct PostToolUseOutput {
    pub universal: UniversalOutput,
    pub should_block: bool,
    pub reason: Option<String>,
    pub invalid_block_reason: Option<String>,
    pub additional_context: Option<String>,
    pub updated_mcp_tool_output: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub(crate) struct PostToolUseFailureOutput {
    pub universal: UniversalOutput,
    pub additional_context: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct PermissionRequestOutput {
    pub universal: UniversalOutput,
    pub behavior: Option<PermissionRequestBehavior>,
    pub updated_input: Option<serde_json::Value>,
    pub updated_permissions: Option<Vec<serde_json::Value>>,
    pub message: Option<String>,
    pub interrupt: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PermissionRequestBehavior {
    Allow,
    Deny,
}
```

#### 6.2 New Parse Functions

```rust
pub(crate) fn parse_pre_tool_use(stdout: &str) -> Option<PreToolUseOutput> {
    let wire: PreToolUseCommandOutputWire = parse_json(stdout)?;

    // Check hookSpecificOutput first (preferred path)
    let (permission_decision, permission_decision_reason, updated_input, additional_context) =
        if let Some(specific) = wire.hook_specific_output {
            let decision = specific.permission_decision.map(|d| match d {
                PreToolUsePermissionDecisionWire::Allow => PreToolUsePermissionDecision::Allow,
                PreToolUsePermissionDecisionWire::Deny => PreToolUsePermissionDecision::Deny,
                PreToolUsePermissionDecisionWire::Ask => PreToolUsePermissionDecision::Ask,
            });
            (decision, specific.permission_decision_reason, specific.updated_input, specific.additional_context)
        }
    // Fallback to deprecated top-level decision (backward compat)
    else if let Some(deprecated) = wire.decision {
            let decision = match deprecated {
                PreToolUseDeprecatedDecisionWire::Approve => PreToolUsePermissionDecision::Allow,
                PreToolUseDeprecatedDecisionWire::Block => PreToolUsePermissionDecision::Deny,
            };
            (Some(decision), wire.reason, None, None)
        } else {
            (None, None, None, None)
        };

    Some(PreToolUseOutput {
        universal: UniversalOutput::from(wire.universal),
        permission_decision,
        permission_decision_reason,
        updated_input,
        additional_context,
    })
}

pub(crate) fn parse_post_tool_use(stdout: &str) -> Option<PostToolUseOutput> {
    let wire: PostToolUseCommandOutputWire = parse_json(stdout)?;
    let should_block = matches!(wire.decision, Some(BlockDecisionWire::Block));
    let invalid_block_reason = if should_block
        && match wire.reason.as_deref() {
            Some(reason) => reason.trim().is_empty(),
            None => true,
        } {
        Some(invalid_block_message("PostToolUse"))
    } else {
        None
    };
    let (additional_context, updated_mcp_tool_output) = wire.hook_specific_output
        .map(|s| (s.additional_context, s.updated_mcp_tool_output))
        .unwrap_or((None, None));
    Some(PostToolUseOutput {
        universal: UniversalOutput::from(wire.universal),
        should_block: should_block && invalid_block_reason.is_none(),
        reason: wire.reason,
        invalid_block_reason,
        additional_context,
        updated_mcp_tool_output,
    })
}

pub(crate) fn parse_post_tool_use_failure(stdout: &str) -> Option<PostToolUseFailureOutput> {
    let wire: PostToolUseFailureCommandOutputWire = parse_json(stdout)?;
    let additional_context = wire.hook_specific_output
        .and_then(|s| s.additional_context);
    Some(PostToolUseFailureOutput {
        universal: UniversalOutput::from(wire.universal),
        additional_context,
    })
}

pub(crate) fn parse_permission_request(stdout: &str) -> Option<PermissionRequestOutput> {
    let wire: PermissionRequestCommandOutputWire = parse_json(stdout)?;
    let (behavior, updated_input, updated_permissions, message, interrupt) =
        wire.hook_specific_output
            .and_then(|s| s.decision)
            .map(|d| {
                let behavior = match d.behavior {
                    PermissionRequestBehaviorWire::Allow => PermissionRequestBehavior::Allow,
                    PermissionRequestBehaviorWire::Deny => PermissionRequestBehavior::Deny,
                };
                (
                    Some(behavior),
                    d.updated_input,
                    d.updated_permissions,
                    d.message,
                    d.interrupt.unwrap_or(false),
                )
            })
            .unwrap_or((None, None, None, None, false));
    Some(PermissionRequestOutput {
        universal: UniversalOutput::from(wire.universal),
        behavior,
        updated_input,
        updated_permissions,
        message,
        interrupt,
    })
}
```

---

## 7. Hooks Crate — Event Modules

### File: `codex-rs/hooks/src/events/mod.rs`

Add module declarations:

```rust
pub(crate) mod common;
pub mod session_start;
pub mod user_prompt_submit;
pub mod stop;
// Phase 1:
pub mod pre_tool_use;
pub mod post_tool_use;
pub mod post_tool_use_failure;
pub mod permission_request;
```

### 7.1 File: `codex-rs/hooks/src/events/pre_tool_use.rs`

#### Request

```rust
#[derive(Debug, Clone)]
pub struct PreToolUseRequest {
    pub session_id: ThreadId,
    pub turn_id: String,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub permission_mode: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub tool_use_id: String,
}
```

#### Outcome

```rust
#[derive(Debug)]
pub struct PreToolUseOutcome {
    pub hook_events: Vec<HookCompletedEvent>,
    pub should_stop: bool,
    pub stop_reason: Option<String>,
    pub permission_decision: Option<PreToolUsePermissionDecision>,
    pub permission_decision_reason: Option<String>,
    pub updated_input: Option<serde_json::Value>,
    pub additional_contexts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreToolUsePermissionDecision {
    Allow,
    Deny,
    Ask,
}
```

#### Run Logic

The `run()` function:
1. Calls `dispatcher::select_handlers(handlers, HookEventName::PreToolUse, Some(&request.tool_name))`
2. Serializes `PreToolUseCommandInput`
3. Dispatches via `dispatcher::execute_handlers`
4. `parse_completed` handles:
   - Exit 0 with JSON → parse `PreToolUseCommandOutputWire`, extract permission decision
   - Exit 0 empty → no-op (allow)
   - Exit 2 → deny with stderr as reason
   - Other exit codes → non-blocking error
5. Aggregates: if any handler returns `deny`, final is deny. If any returns `ask`, final is ask (unless another denied). `allow` only if all agree.

#### Exit Code 2 Behavior

Exit code 2 blocks the tool call (deny). stderr text becomes the denial reason shown to Claude.

#### Tests

- `exit_0_empty_allows_tool_call`
- `exit_0_json_allow_decision`
- `exit_0_json_deny_decision_with_reason`
- `exit_0_json_ask_decision`
- `exit_0_json_deny_without_reason_uses_default_message`
- `exit_0_json_updated_input_modifies_tool_params`
- `exit_0_json_additional_context_injected`
- `exit_2_denies_with_stderr`
- `exit_2_without_stderr_still_denies`
- `exit_1_non_blocking_error`
- `continue_false_overrides_allow`
- `deprecated_approve_maps_to_allow`
- `deprecated_block_maps_to_deny`
- `multiple_handlers_deny_wins_over_allow`
- `matcher_filters_by_tool_name`
- `matcher_regex_mcp_tools`

### 7.2 File: `codex-rs/hooks/src/events/post_tool_use.rs`

#### Request

```rust
#[derive(Debug, Clone)]
pub struct PostToolUseRequest {
    pub session_id: ThreadId,
    pub turn_id: String,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub permission_mode: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub tool_response: serde_json::Value,
    pub tool_use_id: String,
}
```

#### Outcome

```rust
#[derive(Debug)]
pub struct PostToolUseOutcome {
    pub hook_events: Vec<HookCompletedEvent>,
    pub should_stop: bool,
    pub stop_reason: Option<String>,
    pub should_block: bool,
    pub block_reason: Option<String>,
    pub additional_contexts: Vec<String>,
    pub updated_mcp_tool_output: Option<serde_json::Value>,
}
```

#### Run Logic

Same pattern as `stop.rs` for `decision:block` handling. Additionally extracts `additional_context` and `updated_mcp_tool_output` from hookSpecificOutput.

Exit code 2: shows stderr to Claude (tool already ran, cannot block retroactively — but stderr is shown as feedback per Claude Code spec. Note: per the spec table, PostToolUse exit 2 is non-blocking — stderr is "shown to Claude"). Implementation: on exit 2, set feedback entry but do NOT set `should_block`.

Wait — re-reading the spec: PostToolUse **cannot block** via exit code 2. "Shows stderr to Claude (tool already ran)." So exit 2 = non-blocking error with stderr as feedback. The JSON `decision:block` IS supported though — it "prompts Claude with the reason."

Correction: `decision:block` on PostToolUse does NOT prevent the tool (it already ran). It feeds back to Claude as a prompt. So `should_block` means "tell Claude about this problem" not "undo the tool."

#### Tests

- `exit_0_empty_no_op`
- `exit_0_json_no_decision`
- `exit_0_json_block_with_reason_feeds_back`
- `exit_0_json_block_without_reason_fails`
- `exit_0_json_additional_context`
- `exit_0_json_updated_mcp_tool_output`
- `exit_2_shows_stderr_to_claude`
- `exit_1_non_blocking_error`
- `continue_false_stops_session`

### 7.3 File: `codex-rs/hooks/src/events/post_tool_use_failure.rs`

#### Request

```rust
#[derive(Debug, Clone)]
pub struct PostToolUseFailureRequest {
    pub session_id: ThreadId,
    pub turn_id: String,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub permission_mode: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub tool_use_id: String,
    pub error: String,
    pub is_interrupt: bool,
}
```

#### Outcome

```rust
#[derive(Debug)]
pub struct PostToolUseFailureOutcome {
    pub hook_events: Vec<HookCompletedEvent>,
    pub should_stop: bool,
    pub stop_reason: Option<String>,
    pub additional_contexts: Vec<String>,
}
```

#### Run Logic

Simplest of the four. No blocking. No decision control. Only `additionalContext` from hookSpecificOutput and universal fields (continue, systemMessage). Exit 2 = non-blocking (shows stderr to Claude as context alongside the error).

#### Tests

- `exit_0_empty_no_op`
- `exit_0_json_additional_context`
- `exit_0_json_system_message_warning`
- `exit_0_continue_false_stops`
- `exit_2_non_blocking_shows_stderr`
- `exit_1_non_blocking_error`

### 7.4 File: `codex-rs/hooks/src/events/permission_request.rs`

#### Request

```rust
#[derive(Debug, Clone)]
pub struct PermissionRequestRequest {
    pub session_id: ThreadId,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub permission_mode: String,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub permission_suggestions: Option<Vec<serde_json::Value>>,
}
```

#### Outcome

```rust
#[derive(Debug)]
pub struct PermissionRequestOutcome {
    pub hook_events: Vec<HookCompletedEvent>,
    pub should_stop: bool,
    pub stop_reason: Option<String>,
    pub behavior: Option<PermissionRequestBehavior>,
    pub updated_input: Option<serde_json::Value>,
    pub updated_permissions: Option<Vec<serde_json::Value>>,
    pub deny_message: Option<String>,
    pub interrupt: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionRequestBehavior {
    Allow,
    Deny,
}
```

#### Run Logic

1. Exit 0 with JSON → parse `PermissionRequestCommandOutputWire`
   - `behavior: allow` → skip the permission dialog, run the tool (optionally with updatedInput and updatedPermissions)
   - `behavior: deny` → deny permission, show `message` to Claude, optionally `interrupt` to stop
2. Exit 2 → deny permission, stderr as denial message
3. Other exit codes → non-blocking error, fall through to normal permission dialog

#### Tests

- `exit_0_empty_no_decision`
- `exit_0_json_allow_skips_dialog`
- `exit_0_json_allow_with_updated_input`
- `exit_0_json_allow_with_updated_permissions`
- `exit_0_json_deny_with_message`
- `exit_0_json_deny_with_interrupt`
- `exit_2_denies_with_stderr`
- `exit_1_non_blocking_falls_through`
- `continue_false_stops`

---

## 8. Hooks Crate — Engine & Registry

### File: `codex-rs/hooks/src/engine/mod.rs`

Add 4 method pairs:

```rust
pub(crate) fn preview_pre_tool_use(&self, request: &PreToolUseRequest) -> Vec<HookRunSummary> { ... }
pub(crate) async fn run_pre_tool_use(&self, request: PreToolUseRequest) -> PreToolUseOutcome { ... }

pub(crate) fn preview_post_tool_use(&self, request: &PostToolUseRequest) -> Vec<HookRunSummary> { ... }
pub(crate) async fn run_post_tool_use(&self, request: PostToolUseRequest) -> PostToolUseOutcome { ... }

pub(crate) fn preview_post_tool_use_failure(&self, request: &PostToolUseFailureRequest) -> Vec<HookRunSummary> { ... }
pub(crate) async fn run_post_tool_use_failure(&self, request: PostToolUseFailureRequest) -> PostToolUseFailureOutcome { ... }

pub(crate) fn preview_permission_request(&self, request: &PermissionRequestRequest) -> Vec<HookRunSummary> { ... }
pub(crate) async fn run_permission_request(&self, request: PermissionRequestRequest) -> PermissionRequestOutcome { ... }
```

### File: `codex-rs/hooks/src/registry.rs`

Mirror 4 method pairs on `Hooks`:

```rust
pub fn preview_pre_tool_use(&self, request: &PreToolUseRequest) -> Vec<HookRunSummary> {
    self.engine.preview_pre_tool_use(request)
}
pub async fn run_pre_tool_use(&self, request: PreToolUseRequest) -> PreToolUseOutcome {
    self.engine.run_pre_tool_use(request).await
}
// ... same for post_tool_use, post_tool_use_failure, permission_request
```

### File: `codex-rs/hooks/src/lib.rs`

Add public re-exports for all new types:

```rust
pub use events::pre_tool_use::PreToolUseOutcome;
pub use events::pre_tool_use::PreToolUsePermissionDecision;
pub use events::pre_tool_use::PreToolUseRequest;
pub use events::post_tool_use::PostToolUseOutcome;
pub use events::post_tool_use::PostToolUseRequest;
pub use events::post_tool_use_failure::PostToolUseFailureOutcome;
pub use events::post_tool_use_failure::PostToolUseFailureRequest;
pub use events::permission_request::PermissionRequestBehavior;
pub use events::permission_request::PermissionRequestOutcome;
pub use events::permission_request::PermissionRequestRequest;
```

---

## 9. Core Integration

### File: `codex-rs/core/src/hook_runtime.rs`

#### 9.1 PreToolUse Integration

```rust
pub(crate) async fn run_pre_tool_use_hooks(
    sess: &Arc<Session>,
    turn_context: &Arc<TurnContext>,
    tool_name: String,
    tool_input: serde_json::Value,
    tool_use_id: String,
) -> PreToolUseHookDisposition {
    let request = orbit_code_hooks::PreToolUseRequest {
        session_id: sess.conversation_id,
        turn_id: turn_context.sub_id.clone(),
        cwd: turn_context.cwd.clone(),
        transcript_path: sess.hook_transcript_path().await,
        permission_mode: hook_permission_mode(turn_context),
        tool_name,
        tool_input,
        tool_use_id,
    };
    let preview_runs = sess.hooks().preview_pre_tool_use(&request);
    if preview_runs.is_empty() {
        return PreToolUseHookDisposition::NoHooks;
    }

    emit_hook_started_events(sess, turn_context, preview_runs).await;
    let outcome = sess.hooks().run_pre_tool_use(request).await;
    emit_hook_completed_events(sess, turn_context, outcome.hook_events).await;

    if outcome.should_stop {
        return PreToolUseHookDisposition::StopSession {
            stop_reason: outcome.stop_reason,
        };
    }

    record_additional_contexts(sess, turn_context, outcome.additional_contexts).await;

    match outcome.permission_decision {
        Some(orbit_code_hooks::PreToolUsePermissionDecision::Deny) => {
            PreToolUseHookDisposition::Deny {
                reason: outcome.permission_decision_reason
                    .unwrap_or_else(|| "Blocked by hook".to_string()),
            }
        }
        Some(orbit_code_hooks::PreToolUsePermissionDecision::Ask) => {
            PreToolUseHookDisposition::Ask {
                reason: outcome.permission_decision_reason,
                updated_input: outcome.updated_input,
            }
        }
        Some(orbit_code_hooks::PreToolUsePermissionDecision::Allow) => {
            PreToolUseHookDisposition::Allow {
                updated_input: outcome.updated_input,
            }
        }
        None => PreToolUseHookDisposition::NoDecision,
    }
}

pub(crate) enum PreToolUseHookDisposition {
    NoHooks,
    NoDecision,
    Allow { updated_input: Option<serde_json::Value> },
    Deny { reason: String },
    Ask { reason: Option<String>, updated_input: Option<serde_json::Value> },
    StopSession { stop_reason: Option<String> },
}
```

#### 9.2 PostToolUse Integration

```rust
pub(crate) async fn run_post_tool_use_hooks(
    sess: &Arc<Session>,
    turn_context: &Arc<TurnContext>,
    tool_name: String,
    tool_input: serde_json::Value,
    tool_response: serde_json::Value,
    tool_use_id: String,
) -> PostToolUseHookOutcome {
    // Build request, preview, run, emit events, record contexts
    // Return block_reason and updated_mcp_tool_output if any
}

pub(crate) struct PostToolUseHookOutcome {
    pub should_stop: bool,
    pub block_reason: Option<String>,
    pub additional_contexts: Vec<String>,
    pub updated_mcp_tool_output: Option<serde_json::Value>,
}
```

#### 9.3 PostToolUseFailure Integration

```rust
pub(crate) async fn run_post_tool_use_failure_hooks(
    sess: &Arc<Session>,
    turn_context: &Arc<TurnContext>,
    tool_name: String,
    tool_input: serde_json::Value,
    tool_use_id: String,
    error: String,
    is_interrupt: bool,
) -> HookRuntimeOutcome {
    // Build request, preview, run, emit events, record contexts
}
```

#### 9.4 PermissionRequest Integration

```rust
pub(crate) async fn run_permission_request_hooks(
    sess: &Arc<Session>,
    turn_context: &Arc<TurnContext>,
    tool_name: String,
    tool_input: serde_json::Value,
    permission_suggestions: Option<Vec<serde_json::Value>>,
) -> PermissionRequestHookDisposition {
    // Build request, preview, run, emit events
    // Return Allow/Deny/NoDecision/StopSession
}

pub(crate) enum PermissionRequestHookDisposition {
    NoHooks,
    NoDecision,
    Allow {
        updated_input: Option<serde_json::Value>,
        updated_permissions: Option<Vec<serde_json::Value>>,
    },
    Deny {
        message: Option<String>,
        interrupt: bool,
    },
    StopSession { stop_reason: Option<String> },
}
```

### File: `codex-rs/core/src/tools/router.rs` (or equivalent tool dispatch location)

The tool execution flow must be modified to call hooks at the right points:

```
1. Claude generates tool call (tool_name, tool_input, tool_use_id)
2. → run_pre_tool_use_hooks()
   - Deny → return error to Claude, skip execution
   - Ask → show permission dialog (wire into existing approval flow)
   - Allow → apply updatedInput if present, skip normal permission check
   - NoDecision/NoHooks → proceed to normal permission flow
3. → Normal permission check (if not already handled by PreToolUse)
   → If permission dialog would show → run_permission_request_hooks()
     - Allow → grant permission, apply updatedInput/updatedPermissions
     - Deny → deny permission, show message to Claude
     - NoDecision → show normal dialog
4. → Execute tool
5. → If success → run_post_tool_use_hooks()
   → If failure → run_post_tool_use_failure_hooks()
```

The exact file to modify depends on how tool dispatch is currently structured. The key integration points are:
- Before tool execution (after params are generated)
- At the permission dialog point
- After tool execution (success path)
- After tool execution (failure path)

---

## 10. Integration Tests

### File: `codex-rs/core/tests/suite/hooks.rs`

Add integration tests following the existing pattern with `TestCodexBuilder` and wiremock:

```rust
#[tokio::test]
async fn pre_tool_use_hook_denies_bash_command() {
    // Python hook script that reads tool_name from stdin JSON
    // Returns {"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"blocked"}}
    // when tool_name == "Bash"
    // Verify: tool call is blocked, Claude receives denial reason
}

#[tokio::test]
async fn pre_tool_use_hook_allows_with_updated_input() {
    // Hook returns allow with updatedInput modifying the command
    // Verify: tool executes with modified input
}

#[tokio::test]
async fn post_tool_use_hook_provides_feedback() {
    // Hook returns decision:block with reason after Write tool
    // Verify: Claude receives the feedback reason
}

#[tokio::test]
async fn permission_request_hook_auto_approves() {
    // Hook returns behavior:allow for Bash tool
    // Verify: no permission dialog shown, tool executes
}

#[tokio::test]
async fn pre_tool_use_matcher_filters_by_tool_name() {
    // Hook configured with matcher "Bash", tool is Write
    // Verify: hook does not fire
}
```

---

## 11. TUI Rendering

### File: `codex-rs/tui/src/chatwidget.rs` (snapshot tests)

Existing hook rendering already displays HookStarted/HookCompleted events. The new events use the same `HookRunSummary` struct, so they render automatically. Verify with new snapshot tests:

- `pre_tool_use_hook_events_render_snapshot`
- `post_tool_use_hook_events_render_snapshot`
- `permission_request_hook_events_render_snapshot`

Mirror in `tui_app_server/`.

---

## 12. Checklist

Every item must be complete before Phase 1 is done:

- [ ] `HookEventName` has 4 new variants in protocol crate
- [ ] `HookScope::ToolCall` added
- [ ] `HookEventNameWire` has 4 new variants
- [ ] 4 `CommandInput` structs in schema.rs
- [ ] 4 `CommandOutputWire` structs with all nested types
- [ ] 8 schema fixtures generated and committed
- [ ] `HookEvents` config has 4 new fields
- [ ] Discovery loops for 4 events with tool-name matchers
- [ ] `select_handlers` routes matchers for all 4 events
- [ ] `scope_for_event` returns `ToolCall` for all 4 events
- [ ] 4 parse functions in output_parser.rs
- [ ] 4 event modules (pre_tool_use.rs, post_tool_use.rs, post_tool_use_failure.rs, permission_request.rs) each with Request, Outcome, run(), preview(), parse_completed(), tests
- [ ] 8 methods on ClaudeHooksEngine (4 preview + 4 run)
- [ ] 8 methods on Hooks registry (4 preview + 4 run)
- [ ] Public re-exports in lib.rs
- [ ] 4 hook runtime functions in core/hook_runtime.rs with disposition enums
- [ ] Tool dispatch flow calls PreToolUse before execution
- [ ] Tool dispatch flow calls PostToolUse/PostToolUseFailure after execution
- [ ] Permission dialog flow calls PermissionRequest hooks
- [ ] Integration tests for all 4 events
- [ ] TUI snapshot tests (tui + tui_app_server)
- [ ] `just fmt` passes
- [ ] `just fix -p codex-hooks` passes
- [ ] `just fix -p codex-core` passes
- [ ] `cargo test -p codex-hooks` passes
- [ ] `cargo test -p codex-core` passes (including new integration tests)
- [ ] `just write-hooks-schema` regenerates fixtures without diff
