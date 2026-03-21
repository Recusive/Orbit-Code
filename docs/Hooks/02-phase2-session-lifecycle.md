# Phase 2 — Session Lifecycle & Agent Hooks

## Scope

Five events covering session end, agent spawning/completion, API errors, and notifications:

| Event | When | Matcher | Can Block? | Decision Control |
|-------|------|---------|-----------|-----------------|
| **SessionEnd** | Session terminates | exit reason | No | None (cleanup only) |
| **SubagentStart** | Subagent spawned | agent type | No | additionalContext injection |
| **SubagentStop** | Subagent finishes | agent type | Yes | Same as Stop (decision:block) |
| **StopFailure** | Turn ends due to API error | error type | No | None (logging only) |
| **Notification** | Claude Code sends notification | notification type | No | additionalContext |

## Dependencies

- Phase 1 must be complete (dispatcher and discovery patterns established for tool-name matchers generalize to agent-type and notification-type matchers).
- SessionEnd requires `SessionStartSource` enum to be extended with `Clear` and `Compact` variants (currently only `Startup` and `Resume`).

---

## 1. Protocol Changes

### File: `codex-rs/protocol/src/protocol.rs`

#### 1.1 Extend `HookEventName`

```rust
pub enum HookEventName {
    // Existing:
    SessionStart,
    UserPromptSubmit,
    Stop,
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    PermissionRequest,
    // Phase 2:
    SessionEnd,
    SubagentStart,
    SubagentStop,
    StopFailure,
    Notification,
}
```

---

## 2. Schema Wire Types

### File: `codex-rs/hooks/src/schema.rs`

#### 2.1 `HookEventNameWire` — add 5 variants

```rust
#[serde(rename = "SessionEnd")]
SessionEnd,
#[serde(rename = "SubagentStart")]
SubagentStart,
#[serde(rename = "SubagentStop")]
SubagentStop,
#[serde(rename = "StopFailure")]
StopFailure,
#[serde(rename = "Notification")]
Notification,
```

#### 2.2 SessionEnd Input

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "session-end.command.input")]
pub(crate) struct SessionEndCommandInput {
    pub session_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "session_end_hook_event_name_schema")]
    pub hook_event_name: String,
    #[schemars(schema_with = "session_end_reason_schema")]
    pub reason: String,
}
```

SessionEnd reason values: `clear`, `resume`, `logout`, `prompt_input_exit`, `bypass_permissions_disabled`, `other`.

SessionEnd has **no output schema** — output and exit code are ignored per spec. However, we still define a minimal output schema for completeness (universal fields only, all ignored):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[schemars(rename = "session-end.command.output")]
pub(crate) struct SessionEndCommandOutputWire {
    #[serde(flatten)]
    pub universal: HookUniversalOutputWire,
}
```

#### 2.3 SubagentStart Input/Output

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "subagent-start.command.input")]
pub(crate) struct SubagentStartCommandInput {
    pub session_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "subagent_start_hook_event_name_schema")]
    pub hook_event_name: String,
    pub agent_id: String,
    pub agent_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[schemars(rename = "subagent-start.command.output")]
pub(crate) struct SubagentStartCommandOutputWire {
    #[serde(flatten)]
    pub universal: HookUniversalOutputWire,
    #[serde(default)]
    pub hook_specific_output: Option<SubagentStartHookSpecificOutputWire>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub(crate) struct SubagentStartHookSpecificOutputWire {
    pub hook_event_name: HookEventNameWire,
    #[serde(default)]
    pub additional_context: Option<String>,
}
```

#### 2.4 SubagentStop Input/Output

SubagentStop uses the same output format as Stop (decision:block + reason):

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "subagent-stop.command.input")]
pub(crate) struct SubagentStopCommandInput {
    pub session_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "subagent_stop_hook_event_name_schema")]
    pub hook_event_name: String,
    #[schemars(schema_with = "permission_mode_schema")]
    pub permission_mode: String,
    pub stop_hook_active: bool,
    pub agent_id: String,
    pub agent_type: String,
    pub agent_transcript_path: NullableString,
    pub last_assistant_message: NullableString,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[schemars(rename = "subagent-stop.command.output")]
pub(crate) struct SubagentStopCommandOutputWire {
    #[serde(flatten)]
    pub universal: HookUniversalOutputWire,
    #[serde(default)]
    pub decision: Option<BlockDecisionWire>,
    #[serde(default)]
    pub reason: Option<String>,
}
```

#### 2.5 StopFailure Input

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "stop-failure.command.input")]
pub(crate) struct StopFailureCommandInput {
    pub session_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "stop_failure_hook_event_name_schema")]
    pub hook_event_name: String,
    #[schemars(schema_with = "stop_failure_error_schema")]
    pub error: String,
    #[serde(default)]
    pub error_details: Option<String>,
    pub last_assistant_message: NullableString,
}
```

StopFailure error values: `rate_limit`, `authentication_failed`, `billing_error`, `invalid_request`, `server_error`, `max_output_tokens`, `unknown`.

StopFailure output and exit code are **ignored** per spec. Minimal output schema:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[schemars(rename = "stop-failure.command.output")]
pub(crate) struct StopFailureCommandOutputWire {
    #[serde(flatten)]
    pub universal: HookUniversalOutputWire,
}
```

#### 2.6 Notification Input/Output

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "notification.command.input")]
pub(crate) struct NotificationCommandInput {
    pub session_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "notification_hook_event_name_schema")]
    pub hook_event_name: String,
    pub message: String,
    #[serde(default)]
    pub title: Option<String>,
    #[schemars(schema_with = "notification_type_schema")]
    pub notification_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[schemars(rename = "notification.command.output")]
pub(crate) struct NotificationCommandOutputWire {
    #[serde(flatten)]
    pub universal: HookUniversalOutputWire,
    #[serde(default)]
    pub hook_specific_output: Option<NotificationHookSpecificOutputWire>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub(crate) struct NotificationHookSpecificOutputWire {
    pub hook_event_name: HookEventNameWire,
    #[serde(default)]
    pub additional_context: Option<String>,
}
```

Notification type values: `permission_prompt`, `idle_prompt`, `auth_success`, `elicitation_dialog`.

#### 2.7 Schema Fixtures

10 new files (input + output for each of 5 events):
- `session-end.command.input.schema.json`
- `session-end.command.output.schema.json`
- `subagent-start.command.input.schema.json`
- `subagent-start.command.output.schema.json`
- `subagent-stop.command.input.schema.json`
- `subagent-stop.command.output.schema.json`
- `stop-failure.command.input.schema.json`
- `stop-failure.command.output.schema.json`
- `notification.command.input.schema.json`
- `notification.command.output.schema.json`

---

## 3. Engine Config

### File: `codex-rs/hooks/src/engine/config.rs`

Add 5 fields to `HookEvents`:

```rust
#[serde(rename = "SessionEnd", default)]
pub session_end: Vec<MatcherGroup>,
#[serde(rename = "SubagentStart", default)]
pub subagent_start: Vec<MatcherGroup>,
#[serde(rename = "SubagentStop", default)]
pub subagent_stop: Vec<MatcherGroup>,
#[serde(rename = "StopFailure", default)]
pub stop_failure: Vec<MatcherGroup>,
#[serde(rename = "Notification", default)]
pub notification: Vec<MatcherGroup>,
```

---

## 4. Discovery

### File: `codex-rs/hooks/src/engine/discovery.rs`

Add 5 discovery loops. Matcher behavior per event:

| Event | Matcher | `effective_matcher` behavior |
|-------|---------|------------------------------|
| SessionEnd | exit reason string | Pass through (regex match on reason) |
| SubagentStart | agent type name | Pass through (regex match on agent_type) |
| SubagentStop | agent type name | Pass through (regex match on agent_type) |
| StopFailure | error type string | Pass through (regex match on error) |
| Notification | notification type | Pass through (regex match on notification_type) |

Update `effective_matcher()`:

```rust
HookEventName::SessionEnd
| HookEventName::SubagentStart
| HookEventName::SubagentStop
| HookEventName::StopFailure
| HookEventName::Notification => matcher,
```

---

## 5. Dispatcher

### File: `codex-rs/hooks/src/engine/dispatcher.rs`

#### 5.1 `select_handlers` — add matcher routing

All 5 events support regex matchers. Add them to the matcher-supporting branch in `select_handlers`.

#### 5.2 `scope_for_event`

```rust
HookEventName::SessionEnd => HookScope::Thread,
HookEventName::SubagentStart | HookEventName::SubagentStop => HookScope::Turn,
HookEventName::StopFailure | HookEventName::Notification => HookScope::Turn,
```

---

## 6. Output Parser

### File: `codex-rs/hooks/src/engine/output_parser.rs`

#### 6.1 New Parse Functions

```rust
// SessionEnd: output is ignored, but define a no-op parser for consistency.
// The event module itself skips parsing.

pub(crate) fn parse_subagent_start(stdout: &str) -> Option<SubagentStartOutput> {
    let wire: SubagentStartCommandOutputWire = parse_json(stdout)?;
    let additional_context = wire.hook_specific_output
        .and_then(|s| s.additional_context);
    Some(SubagentStartOutput {
        universal: UniversalOutput::from(wire.universal),
        additional_context,
    })
}

pub(crate) fn parse_subagent_stop(stdout: &str) -> Option<StopOutput> {
    // Reuse StopOutput — SubagentStop uses the same output format as Stop.
    let wire: SubagentStopCommandOutputWire = parse_json(stdout)?;
    let should_block = matches!(wire.decision, Some(BlockDecisionWire::Block));
    let invalid_block_reason = if should_block
        && match wire.reason.as_deref() {
            Some(reason) => reason.trim().is_empty(),
            None => true,
        } {
        Some(invalid_block_message("SubagentStop"))
    } else {
        None
    };
    Some(StopOutput {
        universal: UniversalOutput::from(wire.universal),
        should_block: should_block && invalid_block_reason.is_none(),
        reason: wire.reason,
        invalid_block_reason,
    })
}

// StopFailure: output is ignored. No parse function needed.

pub(crate) fn parse_notification(stdout: &str) -> Option<NotificationOutput> {
    let wire: NotificationCommandOutputWire = parse_json(stdout)?;
    let additional_context = wire.hook_specific_output
        .and_then(|s| s.additional_context);
    Some(NotificationOutput {
        universal: UniversalOutput::from(wire.universal),
        additional_context,
    })
}
```

New output structs:

```rust
#[derive(Debug, Clone)]
pub(crate) struct SubagentStartOutput {
    pub universal: UniversalOutput,
    pub additional_context: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct NotificationOutput {
    pub universal: UniversalOutput,
    pub additional_context: Option<String>,
}
```

---

## 7. Event Modules

### File: `codex-rs/hooks/src/events/mod.rs`

```rust
pub mod session_end;
pub mod subagent_start;
pub mod subagent_stop;
pub mod stop_failure;
pub mod notification;
```

### 7.1 `session_end.rs`

#### Request

```rust
#[derive(Debug, Clone)]
pub struct SessionEndRequest {
    pub session_id: ThreadId,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub reason: SessionEndReason,
}

#[derive(Debug, Clone, Copy)]
pub enum SessionEndReason {
    Clear,
    Resume,
    Logout,
    PromptInputExit,
    BypassPermissionsDisabled,
    Other,
}

impl SessionEndReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clear => "clear",
            Self::Resume => "resume",
            Self::Logout => "logout",
            Self::PromptInputExit => "prompt_input_exit",
            Self::BypassPermissionsDisabled => "bypass_permissions_disabled",
            Self::Other => "other",
        }
    }
}
```

#### Outcome

```rust
#[derive(Debug)]
pub struct SessionEndOutcome {
    pub hook_events: Vec<HookCompletedEvent>,
    // No decision fields — SessionEnd cannot block.
}
```

#### Run Logic

- Matcher matches on `reason.as_str()`
- Output and exit code are **ignored** — the hook runs for side effects only
- Default timeout: **1.5 seconds** (per Claude Code spec)
- Respect `CLAUDE_CODE_SESSIONEND_HOOKS_TIMEOUT_MS` env var override
- Per-hook `timeout` capped by the session-end timeout

#### Tests

- `exit_0_completes_normally`
- `exit_2_ignored`
- `timeout_does_not_block_shutdown`
- `matcher_filters_by_reason`

### 7.2 `subagent_start.rs`

#### Request

```rust
#[derive(Debug, Clone)]
pub struct SubagentStartRequest {
    pub session_id: ThreadId,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub agent_id: String,
    pub agent_type: String,
}
```

#### Outcome

```rust
#[derive(Debug)]
pub struct SubagentStartOutcome {
    pub hook_events: Vec<HookCompletedEvent>,
    pub should_stop: bool,
    pub stop_reason: Option<String>,
    pub additional_contexts: Vec<String>,
}
```

#### Run Logic

- Matcher matches on `agent_type`
- Cannot block subagent creation
- Can inject `additionalContext` into subagent
- Exit 2: non-blocking, stderr shown to user only
- Exit 0 with JSON: extract additionalContext, system_message, continue

#### Tests

- `exit_0_injects_context`
- `exit_0_no_output_no_op`
- `exit_2_non_blocking`
- `continue_false_stops`
- `matcher_filters_by_agent_type`

### 7.3 `subagent_stop.rs`

#### Request

```rust
#[derive(Debug, Clone)]
pub struct SubagentStopRequest {
    pub session_id: ThreadId,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub permission_mode: String,
    pub stop_hook_active: bool,
    pub agent_id: String,
    pub agent_type: String,
    pub agent_transcript_path: Option<PathBuf>,
    pub last_assistant_message: Option<String>,
}
```

#### Outcome

Same as `StopOutcome` (reuse or define identical):

```rust
#[derive(Debug)]
pub struct SubagentStopOutcome {
    pub hook_events: Vec<HookCompletedEvent>,
    pub should_stop: bool,
    pub stop_reason: Option<String>,
    pub should_block: bool,
    pub block_reason: Option<String>,
    pub continuation_prompt: Option<String>,
}
```

#### Run Logic

Identical to `stop.rs` — `decision:block` prevents the subagent from stopping, exit code 2 with stderr blocks, etc. Matcher matches on `agent_type`.

#### Tests

- `block_decision_prevents_subagent_from_stopping`
- `exit_2_blocks_with_stderr`
- `continue_false_overrides_block`
- `matcher_filters_by_agent_type`
- `stop_hook_active_passed_in_input`

### 7.4 `stop_failure.rs`

#### Request

```rust
#[derive(Debug, Clone)]
pub struct StopFailureRequest {
    pub session_id: ThreadId,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub error: StopFailureError,
    pub error_details: Option<String>,
    pub last_assistant_message: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum StopFailureError {
    RateLimit,
    AuthenticationFailed,
    BillingError,
    InvalidRequest,
    ServerError,
    MaxOutputTokens,
    Unknown,
}

impl StopFailureError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RateLimit => "rate_limit",
            Self::AuthenticationFailed => "authentication_failed",
            Self::BillingError => "billing_error",
            Self::InvalidRequest => "invalid_request",
            Self::ServerError => "server_error",
            Self::MaxOutputTokens => "max_output_tokens",
            Self::Unknown => "unknown",
        }
    }
}
```

#### Outcome

```rust
#[derive(Debug)]
pub struct StopFailureOutcome {
    pub hook_events: Vec<HookCompletedEvent>,
    // No decision fields — output and exit code are ignored.
}
```

#### Run Logic

- Matcher matches on `error.as_str()`
- Output and exit code are **ignored**
- Runs for logging/alerting only
- Does not block or affect session flow

#### Tests

- `hook_runs_and_output_ignored`
- `matcher_filters_by_error_type`
- `exit_2_ignored`

### 7.5 `notification.rs`

#### Request

```rust
#[derive(Debug, Clone)]
pub struct NotificationRequest {
    pub session_id: ThreadId,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub message: String,
    pub title: Option<String>,
    pub notification_type: String,
}
```

Notification type values: `permission_prompt`, `idle_prompt`, `auth_success`, `elicitation_dialog`.

#### Outcome

```rust
#[derive(Debug)]
pub struct NotificationOutcome {
    pub hook_events: Vec<HookCompletedEvent>,
    pub should_stop: bool,
    pub stop_reason: Option<String>,
    pub additional_contexts: Vec<String>,
}
```

#### Run Logic

- Matcher matches on `notification_type`
- Cannot block notifications
- Can add additionalContext to conversation
- Exit 2: non-blocking, stderr shown to user only

#### Tests

- `exit_0_injects_context`
- `exit_2_non_blocking`
- `matcher_filters_by_notification_type`
- `continue_false_stops`

---

## 8. Engine & Registry

### File: `codex-rs/hooks/src/engine/mod.rs`

Add 5 method pairs (preview + run) for SessionEnd, SubagentStart, SubagentStop, StopFailure, Notification.

### File: `codex-rs/hooks/src/registry.rs`

Mirror 5 method pairs on `Hooks`.

### File: `codex-rs/hooks/src/lib.rs`

Public re-exports for all new types.

---

## 9. Extend SessionStartSource

### File: `codex-rs/hooks/src/events/session_start.rs`

Add missing variants:

```rust
pub enum SessionStartSource {
    Startup,
    Resume,
    Clear,    // new: /clear
    Compact,  // new: auto or manual compaction
}
```

This is a Phase 1 prerequisite gap — fix it in Phase 2 since it's non-breaking and the existing consumers only use Startup/Resume.

---

## 10. Core Integration

### 10.1 SessionEnd

Find the session teardown path in `codex-rs/core/`. The session end hook must fire during session cleanup:

```rust
pub(crate) async fn run_session_end_hooks(
    sess: &Arc<Session>,
    reason: SessionEndReason,
) {
    let request = SessionEndRequest {
        session_id: sess.conversation_id,
        cwd: sess.cwd().clone(),
        transcript_path: sess.hook_transcript_path().await,
        reason,
    };
    let preview_runs = sess.hooks().preview_session_end(&request);
    // Emit started events, run hooks, emit completed events.
    // Apply timeout cap from CLAUDE_CODE_SESSIONEND_HOOKS_TIMEOUT_MS.
    // Do NOT block shutdown on hook failure.
}
```

Integration points:
- `/clear` command handler → reason `Clear`
- Session resume/switch → reason `Resume`
- User exits prompt input → reason `PromptInputExit`
- Normal exit → reason `Other`

### 10.2 SubagentStart / SubagentStop

Find the agent spawning code in `codex-rs/core/src/agent/`. Wire hooks at:
- Agent spawn point → `run_subagent_start_hooks()` before the subagent begins
- Agent completion point → `run_subagent_stop_hooks()` when subagent finishes

SubagentStop can block (prevent agent from stopping). The agent loop must check the outcome and continue if blocked.

### 10.3 StopFailure

Find the API error handling path. When a turn fails due to API error (429, auth failure, etc.), fire StopFailure instead of Stop:

```rust
pub(crate) async fn run_stop_failure_hooks(
    sess: &Arc<Session>,
    error: StopFailureError,
    error_details: Option<String>,
    last_assistant_message: Option<String>,
) {
    // Fire and forget — output ignored.
}
```

### 10.4 Notification

Find notification dispatch in the codebase. When a notification is sent (permission prompt, idle prompt, etc.), fire the Notification hook:

```rust
pub(crate) async fn run_notification_hooks(
    sess: &Arc<Session>,
    turn_context: &Arc<TurnContext>,
    message: String,
    title: Option<String>,
    notification_type: String,
) -> HookRuntimeOutcome {
    // Build request, preview, run, emit events, record contexts.
}
```

---

## 11. Integration Tests

### File: `codex-rs/core/tests/suite/hooks.rs`

```rust
#[tokio::test]
async fn session_end_hook_fires_on_exit() { ... }

#[tokio::test]
async fn session_end_hook_timeout_capped() { ... }

#[tokio::test]
async fn subagent_start_hook_injects_context() { ... }

#[tokio::test]
async fn subagent_stop_hook_blocks_completion() { ... }

#[tokio::test]
async fn stop_failure_hook_fires_on_api_error() { ... }

#[tokio::test]
async fn notification_hook_fires_on_permission_prompt() { ... }
```

---

## 12. Checklist

- [ ] `HookEventName` has 5 new variants
- [ ] `HookEventNameWire` has 5 new variants
- [ ] 5 `CommandInput` structs
- [ ] 5 `CommandOutputWire` structs
- [ ] 10 schema fixtures generated and committed
- [ ] `HookEvents` config has 5 new fields
- [ ] Discovery loops for 5 events
- [ ] `select_handlers` routes matchers for all 5 events
- [ ] `scope_for_event` returns correct scope for all 5 events
- [ ] Parse functions for SubagentStart, SubagentStop, Notification
- [ ] 5 event modules with Request, Outcome, run(), preview(), tests
- [ ] SessionEnd timeout handling (1.5s default, env var override)
- [ ] `SessionStartSource::Clear` and `SessionStartSource::Compact` added
- [ ] 10 methods on ClaudeHooksEngine
- [ ] 10 methods on Hooks registry
- [ ] Public re-exports in lib.rs
- [ ] Core integration: SessionEnd wired into session teardown
- [ ] Core integration: SubagentStart/Stop wired into agent lifecycle
- [ ] Core integration: StopFailure wired into API error handling
- [ ] Core integration: Notification wired into notification dispatch
- [ ] Integration tests for all 5 events
- [ ] TUI snapshot tests
- [ ] `just fmt`, `just fix`, `cargo test` all pass
