# Orbit Code — Testing Guide

> **Last Updated:** 2026-03-21
> **Scope:** Complete testing reference for the `codex-rs/` Rust workspace
> **Audience:** Developers and AI agents building features, adding model providers, modifying the TUI, or changing backend behavior

---

## Overview

This codebase has three testing systems, each for a different concern:

| System | Purpose | Tools | Command |
|---|---|---|---|
| **Unit tests** | Does this function/struct behave correctly? | `#[test]`, `pretty_assertions` | `cargo test -p <crate>` |
| **Integration tests** | Does the full agent loop work end-to-end? | `wiremock`, `TestCodexBuilder`, `ResponseMock` | `cargo test -p orbit-code-core` |
| **Snapshot tests** | Does the TUI render correctly? | `insta`, ratatui `Buffer` | `cargo test -p orbit-code-tui` |

**Rule of thumb:**
- Testing a function → unit test
- Testing a flow (user message → model call → tool execution → response) → integration test
- Testing what the terminal looks like → snapshot test

---

## 1. Unit Tests

### Convention: Sibling `*_tests.rs` Files

This repo does **NOT** use inline `#[cfg(test)] mod tests {}` blocks. Unit tests live in **sibling files**:

```
core/src/
├── auth.rs                    # Implementation
├── auth_tests.rs              # Tests for auth.rs
├── model_provider_info.rs     # Implementation
├── model_provider_info_tests.rs  # Tests
├── config/
│   ├── mod.rs
│   ├── config_tests.rs
│   ├── types_tests.rs
│   ├── schema_tests.rs
│   └── permissions_tests.rs
```

### When to Write Unit Tests

- Struct serialization/deserialization (serde roundtrips)
- Config validation and parsing
- Pure functions with clear input → output
- Error type behavior (`.is_retryable()`, `.to_protocol_error()`)
- Provider config construction and URL building

### Example: Testing a New Model Provider Config

```rust
// model_provider_info_tests.rs

#[test]
fn open_router_provider_deserializes_from_toml() {
    let toml_str = r#"
        name = "Open Router"
        wire_api = "responses"
        base_url = "https://openrouter.ai/api/v1"
        env_key = "OPENROUTER_API_KEY"
    "#;
    let provider: ModelProviderInfo = toml::from_str(toml_str).expect("valid TOML");
    assert_eq!(provider.name, "Open Router");
    assert_eq!(provider.wire_api, WireApi::Responses);
    assert_eq!(provider.env_key, Some("OPENROUTER_API_KEY".into()));
}

#[test]
fn open_router_url_construction() {
    let provider = ModelProviderInfo {
        base_url: Some("https://openrouter.ai/api/v1".into()),
        ..Default::default()
    };
    assert_eq!(
        provider.responses_endpoint(),
        "https://openrouter.ai/api/v1/responses"
    );
}
```

### Rules

- Use `pretty_assertions::assert_eq!` in every test module (workspace convention)
- Compare entire objects with `assert_eq!`, not individual fields
- Use `expect("descriptive message")` instead of `unwrap()` in tests
- No `#[allow(...)]` annotations — fix the issue even in tests

### Running

```bash
cargo test -p orbit-code-core                          # All tests in core
cargo test -p orbit-code-core -- model_provider        # Tests matching "model_provider"
cargo test -p orbit-code-core -- test_name --exact     # One specific test
cargo test -p orbit-code-config                        # Config crate tests
```

---

## 2. Integration Tests (Backend / Agent Logic)

### When to Use

Any time you need to verify that the **agent system works end-to-end**:
- Adding a new model provider (Open Router, VLLM, etc.)
- Changing how tool execution works
- Modifying auth flows or token refresh
- Changing how the agent handles approval requests
- Testing error recovery and retry behavior

### Architecture

Integration tests use three components together:

```
┌──────────────────────┐     ┌─────────────────────┐     ┌──────────────────┐
│  TestCodexBuilder    │────▶│  Core Engine         │────▶│  MockServer      │
│  (configures agent)  │     │  (Session/CodexThread)│     │  (fake model API)│
│                      │     │                      │◀────│                  │
│  .with_model()       │     │  Op → EventMsg loop  │     │  SSE responses   │
│  .with_auth()        │     │                      │     │  Request capture  │
│  .with_config()      │     └─────────────────────┘     └──────────────────┘
└──────────────────────┘              │
                                      ▼
                              ┌─────────────────────┐
                              │  ResponseMock        │
                              │  (captured requests) │
                              │                      │
                              │  .single_request()   │
                              │  .body_json()        │
                              │  .header("auth")     │
                              └─────────────────────┘
```

### Test File Organization

All integration tests live under a single binary:

```
core/tests/
├── all.rs                    # Entry point — mod suite;
├── suite/
│   ├── mod.rs                # Declares all 85+ test modules
│   ├── client.rs             # Model client tests
│   ├── tools.rs              # Tool execution tests
│   ├── auth_refresh.rs       # Auth flow tests
│   ├── models_etag_responses.rs  # Model catalog tests
│   └── ... (85+ modules)
└── common/                   # core_test_support library crate
    ├── lib.rs                # Test macros, wait helpers, config loaders
    ├── test_codex.rs         # TestCodexBuilder fluent API
    ├── responses.rs          # ResponseMock, ev_* event constructors
    └── streaming_sse.rs      # SSE test server
```

**Adding a new integration test:**
1. Create `core/tests/suite/my_feature.rs`
2. Add `mod my_feature;` to `core/tests/suite/mod.rs`
3. Never create a new top-level `tests/*.rs` file — everything goes through `all.rs`

### Step-by-Step: Writing an Integration Test

#### Step 1: Set Up the Mock Server

```rust
use wiremock::MockServer;
use core_test_support::responses::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn open_router_sends_correct_auth_header() -> anyhow::Result<()> {
    // wiremock starts a local HTTP server on a random port
    let server = MockServer::start().await;
```

#### Step 2: Mount Fake API Responses

Tell the mock server what to return when the agent calls it:

```rust
    // This mounts a handler at POST /v1/responses that returns
    // an SSE stream simulating a model response
    let mock = mount_sse_once(&server, sse(vec![
        ev_response_created("resp-1"),
        ev_assistant_message("msg-1", "Hello from Open Router"),
        ev_completed("resp-1"),
    ])).await;
```

**Available mount functions:**

| Function | Use Case |
|---|---|
| `mount_sse_once(&server, body)` | Single API call → single response |
| `mount_sse_once_match(&server, matcher, body)` | Respond only when request matches a condition |
| `mount_sse_sequence(&server, vec![body1, body2])` | Multi-turn: first call gets body1, second gets body2 |
| `mount_compact_json_once(&server, body)` | Non-streaming JSON endpoint |
| `mount_models_once_with_etag(&server, models, etag)` | Mock `/v1/models` catalog |

**Available SSE event constructors (`ev_*`):**

| Constructor | What It Simulates |
|---|---|
| `ev_response_created(id)` | Model starts a response |
| `ev_assistant_message(id, text)` | Model sends text |
| `ev_function_call(call_id, tool, args_json)` | Model wants to call a tool |
| `ev_tool_call_output(call_id, output)` | Tool execution result |
| `ev_completed(id)` | Response finished |
| `ev_completed_with_tokens(id, total)` | Response finished with token count |
| `ev_local_shell_call(call_id, status, cmd)` | Shell tool call |

#### Step 3: Build the Test Session

```rust
    use core_test_support::test_codex;
    use orbit_code_core::auth::CodexAuth;

    let test = test_codex()
        .with_auth(CodexAuth::create_dummy_api_key_auth_for_testing("or-key-123"))
        .with_model("openrouter/llama-3-70b")
        .with_config(|config| {
            // Override provider config to point at mock server
            config.model_provider = ModelProviderInfo {
                name: "Open Router".into(),
                base_url: Some(format!("{}/v1", server.uri())),
                env_key: Some("OPENROUTER_API_KEY".into()),
                wire_api: WireApi::Responses,
                supports_websockets: false,
                ..built_in_model_providers(None)["openai"].clone()
            };
        })
        .build(&server).await?;
```

**`TestCodexBuilder` methods:**

| Method | Purpose |
|---|---|
| `.with_model("name")` | Set which model the agent uses |
| `.with_auth(auth)` | Set authentication credentials |
| `.with_config(\|c\| { ... })` | Mutate the Config struct |
| `.with_home(Arc<TempDir>)` | Isolate home directory |
| `.with_pre_build_hook(\|path\| { ... })` | Run setup before session starts |
| `.with_user_shell(Shell)` | Override the detected shell |
| `.build(&server)` | Build session with HTTP mock |
| `.build_with_streaming_server(&server)` | Build with SSE server |
| `.build_with_websocket_server(&server)` | Build with WebSocket server |

#### Step 4: Submit a User Turn

```rust
    use orbit_code_protocol::protocol::Op;
    use orbit_code_protocol::user_input::UserInput;

    test.codex.submit(Op::UserTurn {
        items: vec![UserInput::text("fix the bug")],
        cwd: test.cwd.path().to_path_buf(),
        ..Default::default()
    }).await?;
```

#### Step 5: Wait for Events

```rust
    use core_test_support::wait_for_event;
    use orbit_code_protocol::protocol::EventMsg;

    // Wait for the agent to finish its turn
    wait_for_event(&test.codex, |ev| {
        matches!(ev, EventMsg::TurnCompleted { .. })
    }).await;
```

**Event waiting functions:**

| Function | Behavior |
|---|---|
| `wait_for_event(codex, predicate)` | Wait up to 1 second for a matching event (preferred) |
| `wait_for_event_with_timeout(codex, predicate, duration)` | Custom timeout |
| `wait_for_event_match(codex, matcher)` | Wait and extract a value from the matching event |

#### Step 6: Assert on Captured Requests

```rust
    // Get the request that was sent to the mock API
    let req = mock.single_request();

    // Assert the model name was sent correctly
    assert_eq!(req.body_json()["model"], "openrouter/llama-3-70b");

    // Assert auth header
    assert_eq!(
        req.header("authorization"),
        Some("Bearer or-key-123".into())
    );

    // Assert request went to the right path
    assert_eq!(req.path(), "/v1/responses");

    Ok(())
}
```

**`ResponsesRequest` assertion methods:**

| Method | What It Checks |
|---|---|
| `.body_json()` | Full request body as `serde_json::Value` |
| `.body_contains_text(s)` | Body contains a string |
| `.input()` | The `input` array sent to the model |
| `.inputs_of_type("message")` | Filter inputs by type |
| `.message_input_texts("user")` | Extract text from user messages |
| `.function_call_output(call_id)` | Tool result the agent sent back |
| `.function_call_output_text(call_id)` | Tool result as plain text |
| `.has_function_call(call_id)` | Was this tool call in the request? |
| `.header("name")` | Read a request header |
| `.path()` | Request URL path |
| `.query_param("name")` | URL query parameter |

### Complete Example: Testing a Model Provider

```rust
// core/tests/suite/open_router.rs

use anyhow::Result;
use core_test_support::*;
use core_test_support::responses::*;
use orbit_code_core::config::built_in_model_providers;
use orbit_code_core::model_provider_info::{ModelProviderInfo, WireApi};
use orbit_code_protocol::protocol::{EventMsg, Op};
use orbit_code_protocol::user_input::UserInput;
use pretty_assertions::assert_eq;
use wiremock::MockServer;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn open_router_full_turn() -> Result<()> {
    let server = MockServer::start().await;

    // Model returns a simple text response
    let mock = mount_sse_once(&server, sse(vec![
        ev_response_created("resp-1"),
        ev_assistant_message("msg-1", "Response via Open Router"),
        ev_completed("resp-1"),
    ])).await;

    let test = test_codex()
        .with_auth(CodexAuth::create_dummy_api_key_auth_for_testing("or-key"))
        .with_model("openrouter/meta-llama/llama-3-70b")
        .with_config(|config| {
            config.model_provider = ModelProviderInfo {
                name: "Open Router".into(),
                base_url: Some(format!("{}/v1", server.uri())),
                env_key: Some("OPENROUTER_API_KEY".into()),
                wire_api: WireApi::Responses,
                supports_websockets: false,
                ..built_in_model_providers(None)["openai"].clone()
            };
        })
        .build(&server).await?;

    // Send a user message
    test.codex.submit(Op::UserTurn {
        items: vec![UserInput::text("hello")],
        cwd: test.cwd.path().to_path_buf(),
        ..Default::default()
    }).await?;

    // Wait for agent to respond
    let msg = wait_for_event_match(&test.codex, |ev| match ev {
        EventMsg::AgentMessage { text, .. } => Some(text.clone()),
        _ => None,
    }).await;

    assert_eq!(msg, "Response via Open Router");

    // Verify the outbound request
    let req = mock.single_request();
    assert_eq!(req.body_json()["model"], "openrouter/meta-llama/llama-3-70b");
    assert_eq!(req.header("authorization"), Some("Bearer or-key".into()));

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn open_router_tool_execution() -> Result<()> {
    let server = MockServer::start().await;

    // Model calls a shell tool, then responds
    let mock = mount_sse_once(&server, sse(vec![
        ev_response_created("resp-1"),
        ev_function_call("call-1", "shell", r#"{"command":["echo","test"]}"#),
        ev_completed("resp-1"),
    ])).await;

    let test = test_codex()
        .with_auth(CodexAuth::create_dummy_api_key_auth_for_testing("or-key"))
        .with_model("openrouter/llama-3-70b")
        .with_config(|config| {
            config.model_provider = ModelProviderInfo {
                name: "Open Router".into(),
                base_url: Some(format!("{}/v1", server.uri())),
                wire_api: WireApi::Responses,
                ..built_in_model_providers(None)["openai"].clone()
            };
        })
        .build(&server).await?;

    test.codex.submit(Op::UserTurn {
        items: vec![UserInput::text("run echo test")],
        cwd: test.cwd.path().to_path_buf(),
        ..Default::default()
    }).await?;

    // Wait for tool execution
    wait_for_event(&test.codex, |ev| {
        matches!(ev, EventMsg::ExecOutput { .. })
    }).await;

    Ok(())
}
```

### Skip Macros

Use these when tests need specific environments:

```rust
skip_if_no_network!();    // Skip when CODEX_SANDBOX_NETWORK_DISABLED=1
skip_if_sandbox!();       // Skip when running under sandbox
skip_if_windows!();       // Skip on Windows
```

### Running

```bash
cargo test -p orbit-code-core                              # All integration + unit tests
cargo test -p orbit-code-core -- suite::open_router        # One test module
cargo test -p orbit-code-core -- open_router_full_turn     # One test by name
cargo nextest run -p orbit-code-core -E 'test(open_router)' # Via nextest (faster)
```

---

## 3. Snapshot Tests (TUI Visual Regression)

### When to Use

Any time you change **how the terminal looks**:
- New or modified chat widget layout
- Status bar changes
- Diff rendering
- Modal dialogs
- Markdown rendering
- Footer or header changes
- Any visual element in the TUI

### How It Works

Tests render TUI widgets into ratatui's in-memory `Buffer` (not a real terminal), convert to a string, and compare against a golden `.snap` file:

```rust
// tui/src/app.rs (test section)
#[test]
fn clear_ui_header() {
    let mut app = App::new(test_config());
    app.clear_transcript();

    // Render to in-memory buffer
    let mut buf = Buffer::empty(Rect::new(0, 0, 45, 5));
    app.render(&mut buf);

    // Compare against saved snapshot
    let rendered = buffer_to_string(&buf);
    insta::assert_snapshot!(rendered);
}
```

The `.snap` file looks like this (actual terminal characters):

```
╭─────────────────────────────────────────────╮
│ >_ Orbit Code (v1.0.0)                      │
│                                             │
│ model:     claude-sonnet   /model to change │
│ directory: /tmp/project                     │
╰─────────────────────────────────────────────╯
```

### Workflow: Building a New TUI Feature

```bash
# 1. Write your widget code in tui/src/
# 2. Write a test that renders it

# 3. Run tests — new test fails (no snapshot yet)
cargo test -p orbit-code-tui

# 4. Review all pending snapshots interactively
#    Shows before/after diff, asks accept/reject for each
cargo insta review -p orbit-code-tui

# 5. Or preview a specific snapshot
cargo insta show -p orbit-code-tui path/to/file.snap.new

# 6. Accept all pending snapshots
cargo insta accept -p orbit-code-tui

# 7. Commit the .snap files alongside your code
git add tui/src/snapshots/*.snap tui/src/my_widget.rs
```

### Workflow: Modifying an Existing TUI Feature

```bash
# 1. Make your change in tui/src/

# 2. Run tests — affected snapshots fail
cargo test -p orbit-code-tui

# 3. Check what changed
cargo insta pending-snapshots -p orbit-code-tui

# 4. Review each change — verify the diff is intentional
cargo insta review -p orbit-code-tui

# 5. Accept the changes
cargo insta accept -p orbit-code-tui

# 6. Commit updated .snap files with your code
```

### Snapshot File Location

Snapshots live in `snapshots/` directories alongside the test files:

```
tui/src/
├── app.rs
├── chatwidget.rs
├── diff_render.rs
├── snapshots/                  # Top-level snapshots
│   ├── codex_tui__app__tests__clear_ui_header.snap
│   ├── codex_tui__diff_render__tests__add_details.snap
│   └── ... (159 files)
├── bottom_pane/
│   ├── mod.rs
│   └── snapshots/              # Sub-module snapshots
│       └── ...
```

### Install insta if Missing

```bash
cargo install cargo-insta
```

---

## 4. When to Use What — Decision Matrix

### Adding a New Model Provider (e.g., Open Router, VLLM)

| What to Test | Test Type | File Location |
|---|---|---|
| Provider config parses from TOML | Unit test | `core/src/model_provider_info_tests.rs` |
| Wire API enum serializes correctly | Unit test | `core/src/model_provider_info_tests.rs` |
| Auth header is built correctly | Unit test | `core/src/auth/*_tests.rs` |
| Full turn works end-to-end | Integration test | `core/tests/suite/open_router.rs` |
| Tool execution works with provider | Integration test | `core/tests/suite/open_router.rs` |
| Error responses are handled | Integration test | `core/tests/suite/open_router.rs` |
| Model appears in TUI model picker | Snapshot test | `tui/src/snapshots/` |

### Adding a New Tool

| What to Test | Test Type | File Location |
|---|---|---|
| Tool spec generates correct schema | Unit test | `core/src/tools/*_tests.rs` |
| Tool executes correctly | Integration test | `core/tests/suite/tools.rs` |
| Tool output renders in TUI | Snapshot test | `tui/src/snapshots/` |

### Changing Auth Flows

| What to Test | Test Type | File Location |
|---|---|---|
| Token parsing/storage | Unit test | `core/src/auth/*_tests.rs` |
| OAuth flow end-to-end | Integration test | `core/tests/suite/auth_refresh.rs` |
| Login UI rendering | Snapshot test | `tui/src/snapshots/` |

### Changing Config Schema

| What to Test | Test Type | File Location |
|---|---|---|
| New field deserializes correctly | Unit test | `core/src/config/config_tests.rs` |
| Schema regeneration | Run command | `just write-config-schema` |
| Config UI rendering | Snapshot test | `tui/src/snapshots/` |

---

## 5. Common Commands Reference

### Running Tests

```bash
# Workspace test runner (uses nextest for speed)
just test                                             # All tests, --no-fail-fast

# Test specific crate
cargo test -p orbit-code-core                         # Core (unit + integration)
cargo test -p orbit-code-tui                          # TUI (unit + snapshots)
cargo test -p orbit-code-config                       # Config
cargo test -p orbit-code-exec                         # Exec (headless)
cargo test -p orbit-code                              # CLI binary

# Run one test by name
cargo test -p orbit-code-core -- test_name            # Partial match
cargo test -p orbit-code-core -- test_name --exact    # Exact match

# Run one test module
cargo test -p orbit-code-core -- suite::client        # Module in integration tests

# Via nextest (faster parallel execution)
cargo nextest run -p orbit-code-core -E 'test(name)'
```

### Snapshot Management

```bash
cargo insta pending-snapshots -p orbit-code-tui       # List pending changes
cargo insta show -p orbit-code-tui path/file.snap.new # Preview one change
cargo insta review -p orbit-code-tui                  # Interactive accept/reject
cargo insta accept -p orbit-code-tui                  # Accept all changes
```

### Schema Regeneration (Required After Type Changes)

```bash
just write-config-schema                              # After ConfigToml changes
just write-app-server-schema                          # After app-server protocol changes
just write-app-server-schema --experimental           # Include experimental surface
just write-hooks-schema                               # After hooks schema changes
```

### Formatting & Linting (Run After Every Change)

```bash
just fmt                                              # Format all Rust code
just fix -p <crate>                                   # Clippy fix for one crate
just fix                                              # Clippy fix for entire workspace (slow)
```

---

## 6. Test Infrastructure Details

### Hermetic Test Isolation

Every integration test gets its own `TempDir` for home directory, config, and state. Tests never touch `~/.orbit` or the real filesystem. The `TestCodexBuilder` handles this automatically.

**Never mutate process environment in tests.** Pass environment-derived flags as parameters, not env vars. This prevents test pollution across parallel test runs.

### Automatic Test Initialization

The test support library uses `#[ctor]` (runs before any test) to:
- Enable deterministic process IDs for reproducible test output
- Set `INSTA_WORKSPACE_ROOT` so snapshot files resolve correctly

### Network-Aware Tests

Some tests need real network access (e.g., testing actual HTTP connections). Use the skip macros:

```rust
skip_if_no_network!();  // Skips when CODEX_SANDBOX_NETWORK_DISABLED=1
```

Most tests mock the network via `wiremock` and don't need real network access.

### Test Dependencies

Standard dev-dependencies available in all test code:

| Crate | Purpose |
|---|---|
| `pretty_assertions` | Better diff output in `assert_eq!` failures |
| `tempfile` | Temporary directories for test isolation |
| `wiremock` | HTTP mock server |
| `insta` | Snapshot testing |
| `tokio` (test features) | Async test runtime |
| `core_test_support` | `TestCodexBuilder`, `ResponseMock`, `wait_for_event`, `ev_*` constructors |
| `assert_cmd` | CLI binary testing (exec, CLI) |
| `predicates` | Assertion predicates for CLI output |

---

## 7. Anti-Patterns — What NOT to Do

| Don't | Do Instead |
|---|---|
| Create a new `tests/my_test.rs` top-level file | Add a module to `tests/suite/mod.rs` |
| Use `#[cfg(test)] mod tests {}` inline | Create a sibling `*_tests.rs` file |
| Use `unwrap()` in tests | Use `expect("descriptive message")` |
| Use raw wiremock matchers directly | Use `mount_sse_once` + `ResponseMock` helpers |
| Mutate env vars in tests | Pass flags as parameters |
| Test individual struct fields | Use `assert_eq!` on entire objects |
| Add `#[allow(dead_code)]` in tests | Delete unused code |
| Skip `just fmt` after changes | Always run `just fmt` |
| Run `just test` for small changes | Run `cargo test -p <crate>` for the specific crate |
| Use `--all-features` routinely | Only when testing feature-gated code |

---

## 8. Wire APIs and Provider Testing

### Currently Supported Wire Formats

The `WireApi` enum in `core/src/model_provider_info.rs` determines how the agent talks to a model:

| WireApi | Protocol | Used By |
|---|---|---|
| `Responses` | OpenAI Responses API (SSE) | OpenAI, Azure, Ollama, Open Router (compatible) |
| `AnthropicMessages` | Anthropic Messages API (SSE) | Claude (API key + OAuth) |

### Adding a New Provider

If the provider uses the OpenAI-compatible Responses API format:
1. Add provider config to `built_in_model_providers()` in `core/src/model_provider_info.rs`
2. Write unit tests for config parsing in `model_provider_info_tests.rs`
3. Write integration tests using `test_codex().with_config()` to override `model_provider`
4. The existing SSE mock infrastructure (`mount_sse_once`, `ev_*`) works as-is

If the provider uses a different wire format:
1. Add a new variant to `WireApi` enum
2. Implement the client in a new crate (like `codex-rs/anthropic/` for Claude)
3. Add dispatch logic in `core/src/client.rs` for the new `WireApi` variant
4. Write unit tests for the new client crate
5. Write integration tests — you may need new `ev_*` constructors for the response format

### Key Files for Provider Work

| File | Purpose |
|---|---|
| `core/src/model_provider_info.rs` | `WireApi` enum, `ModelProviderInfo` struct |
| `core/src/model_provider_info_tests.rs` | Unit tests for provider config |
| `core/src/client.rs` | `ModelClient` — dispatches to provider by `WireApi` |
| `core/tests/common/responses.rs` | Mock infrastructure, `ev_*` constructors |
| `core/tests/common/test_codex.rs` | `TestCodexBuilder` |
| `anthropic/src/` | Anthropic-specific client (reference for new providers) |

---

## 9. Testing Against Real APIs (Live Integration Tests)

Mock tests verify your code sends the right requests and handles responses correctly. But before shipping a new provider, you need to confirm it works against the **actual API**. This section covers how to run live tests.

### When to Use Real API Tests

- **After mock tests pass** — live tests are the final validation, not the first
- **When adding a new provider** — confirm the real API accepts your request format
- **When debugging a production issue** — reproduce with the real endpoint
- **When the API response format might differ from docs** — SSE chunking, header casing, error shapes

### Running the CLI Against a Real Provider

The simplest live test is running the actual binary:

```bash
# Test with OpenAI
OPENAI_API_KEY=sk-... just codex exec "say hello" --quiet

# Test with Anthropic (API key)
ANTHROPIC_API_KEY=sk-ant-... just codex exec -m claude-sonnet-4 "say hello" --quiet

# Test with Anthropic (API key — must login first)
just codex login --provider anthropic
just codex exec -m claude-sonnet-4 "say hello" --quiet

# Test with a local model (Ollama)
just codex exec -m ollama/llama3 "say hello" --quiet

# Test with a custom provider via config
just codex exec -m openrouter/llama-3-70b "say hello" --quiet
```

### Writing Live Integration Tests

For automated live tests, use the same `TestCodexBuilder` but point at the real API instead of a mock. Gate these behind an environment variable so they don't run in CI:

```rust
// core/tests/suite/open_router_live.rs

/// Live tests — only run when OPENROUTER_API_KEY is set.
/// These hit the real API and cost money. Do not run in CI.

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn live_open_router_responds() -> Result<()> {
    // Skip if no API key
    let api_key = match std::env::var("OPENROUTER_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            eprintln!("Skipping: OPENROUTER_API_KEY not set");
            return Ok(());
        }
    };

    let test = test_codex()
        .with_auth(CodexAuth::create_api_key_auth(api_key))
        .with_model("openrouter/meta-llama/llama-3-70b")
        .with_config(|config| {
            config.model_provider = ModelProviderInfo {
                name: "Open Router".into(),
                base_url: Some("https://openrouter.ai/api/v1".into()),
                env_key: Some("OPENROUTER_API_KEY".into()),
                wire_api: WireApi::Responses,
                ..Default::default()
            };
        })
        .build_without_mock().await?;

    test.codex.submit(Op::UserTurn {
        items: vec![UserInput::text("respond with exactly: LIVE_TEST_OK")],
        cwd: test.cwd.path().to_path_buf(),
        ..Default::default()
    }).await?;

    let msg = wait_for_event_match(&test.codex, |ev| match ev {
        EventMsg::AgentMessage { text, .. } => Some(text.clone()),
        _ => None,
    }).await;

    assert!(msg.contains("LIVE_TEST_OK"), "Got: {msg}");
    Ok(())
}
```

### Running Live Tests

```bash
# Run only live tests for a specific provider
OPENROUTER_API_KEY=or-... cargo test -p orbit-code-core -- suite::open_router_live

# Run with verbose output to see request/response details
OPENROUTER_API_KEY=or-... cargo test -p orbit-code-core -- suite::open_router_live --nocapture

# Run all live tests (all providers)
OPENAI_API_KEY=sk-... ANTHROPIC_API_KEY=sk-ant-... OPENROUTER_API_KEY=or-... \
  cargo test -p orbit-code-core -- live
```

### Live Test Conventions

| Rule | Reason |
|---|---|
| Gate behind env var check | Never fail in CI when keys aren't set |
| Use cheap models | Don't burn quota on expensive models for basic connectivity |
| Use deterministic prompts | "respond with exactly: X" gives predictable output |
| Set short `max_tokens` | Limit cost per test run |
| Don't test tool execution live | Tools run locally — mock tests cover this. Live tests verify API connectivity only |
| Keep live tests in separate files | `*_live.rs` suffix makes it clear which tests hit real APIs |

### Testing the Full TUI Against a Real API

For manual testing of the full interactive experience:

```bash
# Launch TUI with a specific provider
just codex                                    # Default provider
just codex -m claude-sonnet-4                 # Anthropic
just codex -m ollama/llama3                   # Local Ollama
just codex -m openrouter/llama-3-70b          # Open Router

# Launch with debug logging to see API calls
RUST_LOG=debug just codex -m claude-sonnet-4

# Launch with state database logging
just log  # In a separate terminal — tails the SQLite log
```

### Recommended Testing Flow for a New Provider

```
1. Unit tests (mock)     — config parsing, URL building, auth headers
                           cargo test -p orbit-code-core -- model_provider

2. Integration tests     — full agent turn with wiremock mock server
   (mock)                  cargo test -p orbit-code-core -- suite::open_router

3. Live connectivity     — single turn against real API
   test                    OPENROUTER_API_KEY=... cargo test -- suite::open_router_live

4. Manual TUI test       — interactive session, try tools, multi-turn
                           just codex -m openrouter/llama-3-70b

5. Headless test         — non-interactive exec mode
                           echo "hello" | just codex exec -m openrouter/llama-3-70b --quiet
```
