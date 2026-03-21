# Phase 3 — Advanced Handler Types & Infrastructure Events

## Scope

Two new handler types and two infrastructure events:

| Component | Type | Description |
|-----------|------|-------------|
| **HTTP hooks** | Handler type | POST to URL, env var interpolation in headers |
| **Async command hooks** | Handler type | Background execution, deliver results on next turn |
| **ConfigChange** | Event | Config file changed during session |
| **InstructionsLoaded** | Event | CLAUDE.md or rules file loaded into context |

## Dependencies

Phase 1 and Phase 2 must be complete. The dispatcher, discovery, and output parser patterns are fully established.

---

## 1. HTTP Hooks

### 1.1 Config

#### File: `codex-rs/hooks/src/engine/config.rs`

Extend `HookHandlerConfig`:

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub(crate) enum HookHandlerConfig {
    #[serde(rename = "command")]
    Command {
        command: String,
        #[serde(default, rename = "timeout", alias = "timeoutSec")]
        timeout_sec: Option<u64>,
        #[serde(default)]
        r#async: bool,
        #[serde(default, rename = "statusMessage")]
        status_message: Option<String>,
    },
    #[serde(rename = "http")]
    Http {
        url: String,
        #[serde(default, rename = "timeout", alias = "timeoutSec")]
        timeout_sec: Option<u64>,
        #[serde(default, rename = "statusMessage")]
        status_message: Option<String>,
        #[serde(default)]
        headers: Option<std::collections::HashMap<String, String>>,
        #[serde(default, rename = "allowedEnvVars")]
        allowed_env_vars: Option<Vec<String>>,
    },
    #[serde(rename = "prompt")]
    Prompt {},
    #[serde(rename = "agent")]
    Agent {},
}
```

### 1.2 ConfiguredHandler

#### File: `codex-rs/hooks/src/engine/mod.rs`

The current `ConfiguredHandler` assumes a shell command. Extend to support HTTP:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum HandlerKind {
    Command { command: String },
    Http {
        url: String,
        headers: std::collections::HashMap<String, String>,
        allowed_env_vars: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConfiguredHandler {
    pub event_name: HookEventName,
    pub matcher: Option<String>,
    pub kind: HandlerKind,
    pub timeout_sec: u64,
    pub status_message: Option<String>,
    pub source_path: PathBuf,
    pub display_order: i64,
}
```

This is a **breaking internal refactor**. Every reference to `handler.command` changes to `handler.kind`. The `command` field accessor becomes:

```rust
impl ConfiguredHandler {
    pub fn command_str(&self) -> Option<&str> {
        match &self.kind {
            HandlerKind::Command { command } => Some(command),
            HandlerKind::Http { .. } => None,
        }
    }

    pub fn handler_type(&self) -> HookHandlerType {
        match &self.kind {
            HandlerKind::Command { .. } => HookHandlerType::Command,
            HandlerKind::Http { .. } => HookHandlerType::Http,
        }
    }
}
```

Update `HookHandlerType` in protocol if `Http` is not already there:

```rust
pub enum HookHandlerType {
    Command,
    Http,    // new
    Prompt,
    Agent,
}
```

### 1.3 Discovery

#### File: `codex-rs/hooks/src/engine/discovery.rs`

In `append_group_handlers`, add the `Http` variant:

```rust
HookHandlerConfig::Http {
    url,
    timeout_sec,
    status_message,
    headers,
    allowed_env_vars,
} => {
    if url.trim().is_empty() {
        warnings.push(format!(
            "skipping empty hook URL in {}",
            source_path.display()
        ));
        continue;
    }
    let timeout_sec = timeout_sec.unwrap_or(30).max(1);
    handlers.push(ConfiguredHandler {
        event_name,
        matcher: matcher.map(ToOwned::to_owned),
        kind: HandlerKind::Http {
            url,
            headers: headers.unwrap_or_default(),
            allowed_env_vars: allowed_env_vars.unwrap_or_default(),
        },
        timeout_sec,
        status_message,
        source_path: source_path.to_path_buf(),
        display_order: *display_order,
    });
    *display_order += 1;
}
```

### 1.4 HTTP Runner

#### New file: `codex-rs/hooks/src/engine/http_runner.rs`

```rust
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;

use reqwest::Client;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::header::HeaderValue;

use super::ConfiguredHandler;
use super::HandlerKind;
use super::command_runner::CommandRunResult;

/// Run an HTTP hook by POSTing the input JSON to the configured URL.
/// Returns a CommandRunResult for compatibility with the existing parse pipeline.
pub(crate) async fn run_http(
    handler: &ConfiguredHandler,
    input_json: &str,
    _cwd: &Path,
) -> CommandRunResult {
    let started_at = chrono::Utc::now().timestamp();
    let started = Instant::now();

    let HandlerKind::Http {
        url,
        headers,
        allowed_env_vars,
    } = &handler.kind
    else {
        return error_result(started_at, started, "handler is not an HTTP hook");
    };

    let client = Client::new();
    let timeout_duration = Duration::from_secs(handler.timeout_sec);

    // Build headers with env var interpolation
    let mut header_map = HeaderMap::new();
    for (key, value_template) in headers {
        let resolved_value = interpolate_env_vars(value_template, allowed_env_vars);
        let Ok(header_name) = key.parse::<HeaderName>() else {
            return error_result(started_at, started, &format!("invalid header name: {key}"));
        };
        let Ok(header_value) = HeaderValue::from_str(&resolved_value) else {
            return error_result(
                started_at,
                started,
                &format!("invalid header value for {key}"),
            );
        };
        header_map.insert(header_name, header_value);
    }

    let response = match tokio::time::timeout(
        timeout_duration,
        client
            .post(url)
            .headers(header_map)
            .header("Content-Type", "application/json")
            .body(input_json.to_string())
            .send(),
    )
    .await
    {
        Ok(Ok(response)) => response,
        Ok(Err(err)) => {
            // Connection failure: non-blocking error
            return CommandRunResult {
                started_at,
                completed_at: chrono::Utc::now().timestamp(),
                duration_ms: elapsed_ms(started),
                exit_code: None,
                stdout: String::new(),
                stderr: String::new(),
                error: Some(format!("HTTP request failed: {err}")),
            };
        }
        Err(_) => {
            // Timeout: non-blocking error
            return CommandRunResult {
                started_at,
                completed_at: chrono::Utc::now().timestamp(),
                duration_ms: elapsed_ms(started),
                exit_code: None,
                stdout: String::new(),
                stderr: String::new(),
                error: Some(format!("HTTP hook timed out after {}s", handler.timeout_sec)),
            };
        }
    };

    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    if status.is_success() {
        // 2xx: success. Body is either empty, plain text, or JSON.
        CommandRunResult {
            started_at,
            completed_at: chrono::Utc::now().timestamp(),
            duration_ms: elapsed_ms(started),
            exit_code: Some(0), // Map 2xx to exit 0 for output parser compatibility
            stdout: body,
            stderr: String::new(),
            error: None,
        }
    } else {
        // Non-2xx: non-blocking error
        CommandRunResult {
            started_at,
            completed_at: chrono::Utc::now().timestamp(),
            duration_ms: elapsed_ms(started),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            error: Some(format!("HTTP hook returned status {status}: {body}")),
        }
    }
}

/// Interpolate `$VAR_NAME` and `${VAR_NAME}` in a string.
/// Only variables listed in `allowed_env_vars` are resolved.
/// References to unlisted variables are replaced with empty strings.
fn interpolate_env_vars(template: &str, allowed_env_vars: &[String]) -> String {
    let mut result = template.to_string();
    // Handle ${VAR_NAME} syntax
    let re_braced = regex::Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}")
        .expect("valid regex");
    result = re_braced
        .replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            if allowed_env_vars.iter().any(|v| v == var_name) {
                std::env::var(var_name).unwrap_or_default()
            } else {
                String::new()
            }
        })
        .to_string();
    // Handle $VAR_NAME syntax (without braces)
    let re_plain = regex::Regex::new(r"\$([A-Za-z_][A-Za-z0-9_]*)")
        .expect("valid regex");
    result = re_plain
        .replace_all(&result, |caps: &regex::Captures| {
            let var_name = &caps[1];
            if allowed_env_vars.iter().any(|v| v == var_name) {
                std::env::var(var_name).unwrap_or_default()
            } else {
                String::new()
            }
        })
        .to_string();
    result
}

fn elapsed_ms(started: Instant) -> i64 {
    started.elapsed().as_millis().try_into().unwrap_or(i64::MAX)
}

fn error_result(started_at: i64, started: Instant, message: &str) -> CommandRunResult {
    CommandRunResult {
        started_at,
        completed_at: chrono::Utc::now().timestamp(),
        duration_ms: elapsed_ms(started),
        exit_code: None,
        stdout: String::new(),
        stderr: String::new(),
        error: Some(message.to_string()),
    }
}
```

**Key difference from command hooks**: HTTP hooks cannot signal blocking errors through status codes alone. To block, they must return 2xx with JSON containing the appropriate decision fields.

### 1.5 Dispatcher — Route to HTTP Runner

#### File: `codex-rs/hooks/src/engine/dispatcher.rs`

Update `execute_handlers` to route based on `HandlerKind`:

```rust
pub(crate) async fn execute_handlers<T>(
    shell: &CommandShell,
    handlers: Vec<ConfiguredHandler>,
    input_json: String,
    cwd: &Path,
    turn_id: Option<String>,
    parse: fn(&ConfiguredHandler, CommandRunResult, Option<String>) -> ParsedHandler<T>,
) -> Vec<ParsedHandler<T>> {
    let results = join_all(handlers.iter().map(|handler| async {
        match &handler.kind {
            HandlerKind::Command { .. } => {
                command_runner::run_command(shell, handler, &input_json, cwd).await
            }
            HandlerKind::Http { .. } => {
                http_runner::run_http(handler, &input_json, cwd).await
            }
        }
    }))
    .await;

    handlers
        .into_iter()
        .zip(results)
        .map(|(handler, result)| parse(&handler, result, turn_id.clone()))
        .collect()
}
```

Update `running_summary` and `completed_summary` to use `handler.handler_type()` instead of hardcoded `HookHandlerType::Command`.

### 1.6 Tests for HTTP Hooks

- `http_hook_posts_json_and_parses_response`
- `http_hook_non_2xx_is_non_blocking_error`
- `http_hook_timeout_is_non_blocking_error`
- `http_hook_connection_failure_is_non_blocking_error`
- `http_hook_env_var_interpolation_in_headers`
- `http_hook_unlisted_env_vars_replaced_with_empty`
- `http_hook_empty_body_is_success`
- `http_hook_plain_text_body_is_context`
- `http_hook_json_body_parsed`

### 1.7 Cargo.toml

Add `reqwest` dependency to `codex-rs/hooks/Cargo.toml`:

```toml
[dependencies]
reqwest = { workspace = true, features = ["json"] }
```

Verify `reqwest` is already in `[workspace.dependencies]` in root `Cargo.toml` (it likely is, since `codex-client` uses it).

---

## 2. Async Command Hooks

### 2.1 ConfiguredHandler Extension

Add an `is_async` flag to the `Command` variant tracking:

```rust
pub(crate) enum HandlerKind {
    Command { command: String, is_async: bool },
    Http { ... },
}
```

### 2.2 Discovery

In `append_group_handlers`, stop skipping async hooks. Instead, record `is_async: true`:

```rust
HookHandlerConfig::Command {
    command,
    timeout_sec,
    r#async,
    status_message,
} => {
    // Remove the async skip warning
    let timeout_sec = timeout_sec.unwrap_or(600).max(1);
    handlers.push(ConfiguredHandler {
        event_name,
        matcher: matcher.map(ToOwned::to_owned),
        kind: HandlerKind::Command {
            command,
            is_async: r#async,
        },
        timeout_sec,
        status_message,
        source_path: source_path.to_path_buf(),
        display_order: *display_order,
    });
    *display_order += 1;
}
```

### 2.3 Async Execution

#### New file: `codex-rs/hooks/src/engine/async_command_runner.rs`

```rust
use std::path::Path;
use std::path::PathBuf;

use tokio::sync::mpsc;

use super::CommandShell;
use super::ConfiguredHandler;
use super::command_runner;

/// Result delivered when an async hook completes.
#[derive(Debug)]
pub(crate) struct AsyncHookResult {
    pub handler_id: String,
    pub system_message: Option<String>,
    pub additional_context: Option<String>,
}

/// Spawn an async hook in the background. Returns immediately.
/// The result is sent via the channel when the hook completes.
pub(crate) fn spawn_async_hook(
    shell: CommandShell,
    handler: ConfiguredHandler,
    input_json: String,
    cwd: PathBuf,
    result_tx: mpsc::UnboundedSender<AsyncHookResult>,
) {
    let handler_id = handler.run_id();
    tokio::spawn(async move {
        let run_result =
            command_runner::run_command(&shell, &handler, &input_json, &cwd).await;

        // Only extract systemMessage and additionalContext from async hooks.
        // Decision fields are ignored (action already completed).
        let (system_message, additional_context) = if run_result.exit_code == Some(0) {
            parse_async_output(&run_result.stdout)
        } else {
            (None, None)
        };

        let _ = result_tx.send(AsyncHookResult {
            handler_id,
            system_message,
            additional_context,
        });
    });
}

fn parse_async_output(stdout: &str) -> (Option<String>, Option<String>) {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return (None, None);
    }
    let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
        return (None, None);
    };
    let system_message = value
        .get("systemMessage")
        .and_then(|v| v.as_str())
        .map(String::from);
    let additional_context = value
        .get("additionalContext")
        .and_then(|v| v.as_str())
        .map(String::from);
    (system_message, additional_context)
}
```

### 2.4 Dispatcher — Split Sync and Async

In `execute_handlers`, partition handlers into sync and async:

```rust
let (sync_handlers, async_handlers): (Vec<_>, Vec<_>) = handlers
    .into_iter()
    .partition(|h| !matches!(&h.kind, HandlerKind::Command { is_async: true, .. }));
```

Execute sync handlers as before. For async handlers, spawn via `spawn_async_hook`. Return only sync results (async results arrive later via channel).

### 2.5 Core Integration

The `Session` struct needs an `mpsc::UnboundedReceiver<AsyncHookResult>` to receive async hook completions. On each new conversation turn, drain the receiver and inject any `system_message` or `additional_context` as developer messages.

### 2.6 Constraints Per Spec

- Only `type: "command"` hooks support `async`. HTTP/prompt/agent hooks ignore the field.
- Async hooks cannot block or return decisions. Decision fields are ignored.
- Results are delivered on the next conversation turn.
- Each firing creates a separate background task. No deduplication.
- Default timeout: same as sync (600s / 10 minutes).

### 2.7 Tests

- `async_hook_spawns_and_delivers_system_message`
- `async_hook_decision_fields_ignored`
- `async_hook_timeout_does_not_block_session`
- `async_hook_results_arrive_on_next_turn`

---

## 3. ConfigChange Event

### 3.1 Protocol

Add `ConfigChange` to `HookEventName`.

### 3.2 Schema

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "config-change.command.input")]
pub(crate) struct ConfigChangeCommandInput {
    pub session_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "config_change_hook_event_name_schema")]
    pub hook_event_name: String,
    #[schemars(schema_with = "config_change_source_schema")]
    pub source: String,
    #[serde(default)]
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[schemars(rename = "config-change.command.output")]
pub(crate) struct ConfigChangeCommandOutputWire {
    #[serde(flatten)]
    pub universal: HookUniversalOutputWire,
    #[serde(default)]
    pub decision: Option<BlockDecisionWire>,
    #[serde(default)]
    pub reason: Option<String>,
}
```

ConfigChange source values: `user_settings`, `project_settings`, `local_settings`, `policy_settings`, `skills`.

### 3.3 Event Module

#### File: `codex-rs/hooks/src/events/config_change.rs`

```rust
#[derive(Debug, Clone)]
pub struct ConfigChangeRequest {
    pub session_id: ThreadId,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub source: String,
    pub file_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct ConfigChangeOutcome {
    pub hook_events: Vec<HookCompletedEvent>,
    pub should_stop: bool,
    pub stop_reason: Option<String>,
    pub should_block: bool,
    pub block_reason: Option<String>,
}
```

**Blocking behavior**: Exit code 2 or `decision:block` blocks the config change from taking effect — **except** `policy_settings` which cannot be blocked (hooks still fire for audit, but blocking is ignored).

### 3.4 Core Integration

Wire into the config file watcher. When a settings file changes:
1. Fire ConfigChange hook
2. If blocked (and source is not `policy_settings`), discard the new config
3. If allowed, apply the new config

---

## 4. InstructionsLoaded Event

### 4.1 Protocol

Add `InstructionsLoaded` to `HookEventName`.

### 4.2 Schema

```rust
#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "instructions-loaded.command.input")]
pub(crate) struct InstructionsLoadedCommandInput {
    pub session_id: String,
    pub transcript_path: NullableString,
    pub cwd: String,
    #[schemars(schema_with = "instructions_loaded_hook_event_name_schema")]
    pub hook_event_name: String,
    pub file_path: String,
    #[schemars(schema_with = "instructions_loaded_memory_type_schema")]
    pub memory_type: String,
    #[schemars(schema_with = "instructions_loaded_reason_schema")]
    pub load_reason: String,
    #[serde(default)]
    pub globs: Option<Vec<String>>,
    #[serde(default)]
    pub trigger_file_path: Option<String>,
    #[serde(default)]
    pub parent_file_path: Option<String>,
}
```

Memory type values: `User`, `Project`, `Local`, `Managed`.
Load reason values: `session_start`, `nested_traversal`, `path_glob_match`, `include`, `compact`.

InstructionsLoaded has **no output schema** — exit code is ignored. Minimal output schema for schema completeness:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
#[schemars(rename = "instructions-loaded.command.output")]
pub(crate) struct InstructionsLoadedCommandOutputWire {
    #[serde(flatten)]
    pub universal: HookUniversalOutputWire,
}
```

### 4.3 Event Module

#### File: `codex-rs/hooks/src/events/instructions_loaded.rs`

```rust
#[derive(Debug, Clone)]
pub struct InstructionsLoadedRequest {
    pub session_id: ThreadId,
    pub cwd: PathBuf,
    pub transcript_path: Option<PathBuf>,
    pub file_path: PathBuf,
    pub memory_type: String,
    pub load_reason: String,
    pub globs: Option<Vec<String>>,
    pub trigger_file_path: Option<PathBuf>,
    pub parent_file_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct InstructionsLoadedOutcome {
    pub hook_events: Vec<HookCompletedEvent>,
    // No decision fields. Cannot block. Runs async for observability.
}
```

**Behavior**: Runs asynchronously. Output is ignored. Cannot block or modify instruction loading. Matcher matches on `load_reason`.

### 4.4 Core Integration

Wire into instruction loading in `codex-rs/core/src/instructions/` or wherever CLAUDE.md files are loaded. Fire asynchronously (don't wait for completion).

---

## 5. Hook Deduplication

Per Claude Code spec: "All matching hooks run in parallel, and identical handlers are deduplicated automatically. Command hooks are deduplicated by command string, and HTTP hooks are deduplicated by URL."

Add deduplication in `execute_handlers`:

```rust
fn deduplicate_handlers(handlers: Vec<ConfiguredHandler>) -> Vec<ConfiguredHandler> {
    let mut seen = std::collections::HashSet::new();
    handlers
        .into_iter()
        .filter(|handler| {
            let key = match &handler.kind {
                HandlerKind::Command { command, .. } => command.clone(),
                HandlerKind::Http { url, .. } => url.clone(),
            };
            seen.insert(key)
        })
        .collect()
}
```

---

## 6. Environment Variables for Hooks

Per spec, hooks receive these environment variables:

| Variable | Description |
|----------|-------------|
| `CLAUDE_PROJECT_DIR` | Project root directory |
| `CLAUDE_ENV_FILE` | SessionStart only: path to write persistent env vars |
| `CLAUDE_CODE_REMOTE` | Set to `"true"` in remote environments |

Ensure `command_runner.rs` sets `CLAUDE_PROJECT_DIR` on the child process. For `CLAUDE_ENV_FILE`, only set it for SessionStart hooks. Source the env file after SessionStart hooks complete.

### File: `codex-rs/hooks/src/engine/command_runner.rs`

```rust
fn build_command(shell: &CommandShell, handler: &ConfiguredHandler) -> Command {
    let mut command = /* existing logic */;
    // Set CLAUDE_PROJECT_DIR from the cwd or project root
    // This must be passed in from the caller or derived from the handler context
    command
}
```

Add a `env_vars: HashMap<String, String>` parameter to `run_command` for per-invocation env vars. For SessionStart, include `CLAUDE_ENV_FILE`.

---

## 7. disableAllHooks Setting

Per spec: `"disableAllHooks": true` in settings disables all hooks. Managed settings hierarchy applies — user/project/local settings cannot disable managed hooks.

### File: `codex-rs/hooks/src/engine/discovery.rs`

During discovery, check each config layer for `disableAllHooks`. If set at a layer, skip hooks from that layer and lower-precedence layers. If set at managed level, skip all hooks.

---

## 8. Checklist

- [ ] `HookHandlerConfig::Http` variant with url, headers, allowedEnvVars
- [ ] `HandlerKind` enum replaces `command: String` field on `ConfiguredHandler`
- [ ] All existing code updated for `HandlerKind` refactor
- [ ] `HookHandlerType::Http` added to protocol
- [ ] `http_runner.rs` created with POST, env var interpolation, timeout handling
- [ ] HTTP non-2xx / timeout / connection failure → non-blocking error
- [ ] Dispatcher routes to http_runner for Http handlers
- [ ] Tests for HTTP hooks (8+ tests)
- [ ] `async_command_runner.rs` created
- [ ] Async hooks spawn background tasks, deliver results via channel
- [ ] Async hook decision fields ignored
- [ ] Session drains async results on next turn
- [ ] Tests for async hooks (4+ tests)
- [ ] ConfigChange event: schema, event module, core integration
- [ ] ConfigChange blocking (except policy_settings)
- [ ] InstructionsLoaded event: schema, event module, core integration
- [ ] InstructionsLoaded runs async, output ignored
- [ ] Hook deduplication by command string / URL
- [ ] `CLAUDE_PROJECT_DIR` set on hook child processes
- [ ] `CLAUDE_ENV_FILE` set for SessionStart hooks
- [ ] `disableAllHooks` setting respected
- [ ] `reqwest` dependency added to hooks crate
- [ ] All schema fixtures generated
- [ ] `just fmt`, `just fix`, `cargo test` all pass
