# Plan: Request `reasoning.content` from Responses API

> **Status:** Done (implemented 2026-03-25)
> **Audit:** `reviews/request-reasoning-content-from-api.audit.md`
> **Depends on:** `show-thinking-tokens-in-tui.md` (done — TUI display layer is implemented)

## Context

GPT-5.4 with xhigh reasoning generates thinking tokens, but they never appear in the TUI. The entire downstream pipeline is wired — `event_mapping.rs` parses content from `ResponseItem::Reasoning`, `items.rs` emits `AgentReasoningRawContentEvent`, and the TUI's new `ThinkingStreamController` (just implemented) renders them in italic magenta. The missing piece: the API request never asks for plaintext reasoning.

In `core/src/client.rs:748`, the `include` parameter is hardcoded to only `["reasoning.encrypted_content"]`. The `show_raw_agent_reasoning` config flag exists but never reaches the API request layer because `ModelClientState` doesn't carry it.

## Approach

Add `show_raw_agent_reasoning: bool` to `ModelClientState` (session-scoped, matching the pattern of `model_verbosity`, `enable_request_compression`, `include_timing_metrics`). When true, also include `"reasoning.content"` in the API request `include` array.

## Files to Modify

### `core/src/client.rs` — 3 edits

1. **`ModelClientState` (line 144)**: Add `show_raw_agent_reasoning: bool` field after `include_timing_metrics`
2. **`ModelClient::new()` (line 265)**: Add `show_raw_agent_reasoning: bool` parameter, pass to struct
3. **`build_responses_request()` (line 748)**: When `show_raw_agent_reasoning` is true, push `"reasoning.content"` to the include vec

```rust
// Line 748 — change from:
let include = if reasoning.is_some() {
    vec!["reasoning.encrypted_content".to_string()]
} else {
    Vec::new()
};

// To:
let include = if reasoning.is_some() {
    let mut inc = vec!["reasoning.encrypted_content".to_string()];
    if self.client.state.show_raw_agent_reasoning {
        inc.push("reasoning.content".to_string());
    }
    inc
} else {
    Vec::new()
};
```

### `core/src/codex.rs` — 1 edit

4. **`ModelClient::new()` call (line 1819)**: Add `config.show_raw_agent_reasoning,` after `Feature::RuntimeMetrics` arg

### Test call sites — pass `false` with `/*show_raw_agent_reasoning*/` comment (Convention 30)

| # | File | Line | Context |
|---|------|------|---------|
| 5 | `core/src/orbit_code_tests.rs` | 228 | `test_model_client_session()` |
| 6 | `core/src/orbit_code_tests.rs` | 2492 | Integration test session #1 — use `config.show_raw_agent_reasoning` |
| 7 | `core/src/orbit_code_tests.rs` | 3286 | Integration test session #2 — use `config.show_raw_agent_reasoning` |
| 8 | `core/src/client_tests.rs` | 18 | `test_model_client()` |
| 9 | `core/tests/suite/client.rs` | 1824 | `chatgpt_auth_sends_correct_request()` |
| 10 | `core/tests/suite/client_websockets.rs` | 1754 | WebSocket test |
| 11-13 | `core/tests/responses_headers.rs` | 89, 202, 315 | Three header test call sites |

All test call sites pass `false` (default behavior) except #6 and #7 which use `config.show_raw_agent_reasoning`.

### Positive test (audit critical issue #1)

14. **`core/tests/suite/client.rs`** — `show_raw_agent_reasoning_includes_reasoning_content` test: creates a session with `show_raw_agent_reasoning = true` via `TestCodexBuilder`, sends a request through a mock server, and asserts the `include` array contains both `"reasoning.encrypted_content"` AND `"reasoning.content"`.

## What Does NOT Change

- No `ConfigToml` / config schema changes (flag already exists)
- No protocol changes
- No TUI changes (already implemented)
- No app-server changes
- No Bazel lockfile changes

## Verification

1. `cargo build -p orbit-code-core`
2. `just fix -p orbit-code-core`
3. `just fmt`
4. `cargo test -p orbit-code-core`
5. E2E: run TUI with `show-raw-agent-reasoning = true`, send prompt to GPT-5.4 xhigh, verify thinking tokens stream live in italic magenta
