# Plan Audit: Stage 3a — Claude Provider (Anthropic Messages API + OAuth)

**Date**: 2026-03-20
**Plan Document**: `docs/migration/03a-claude-provider.md`
**Branch**: main

## Plan Summary

This plan adds full Claude model support to Orbit Code by creating a new `orbit-anthropic` crate implementing the Anthropic Messages API with SSE streaming, extending the `WireApi` enum with an `AnthropicMessages` variant, adding Anthropic OAuth PKCE flow to the login crate, and wiring dispatch in `core::client.rs`. Users would authenticate via API key or Claude Pro/Max OAuth (browser code-paste flow).

## Files Reviewed

| File | Role | Risk |
|------|------|------|
| `codex-rs/core/src/model_provider_info.rs` | `WireApi` enum, provider registry, `ModelProviderInfo` struct | High |
| `codex-rs/core/src/client.rs` | Model client, stream dispatch, WebSocket/SSE transport | High |
| `codex-rs/core/src/auth.rs` | `AuthMode`, `CodexAuth`, `AuthManager`, token refresh | High |
| `codex-rs/core/src/api_bridge.rs` | Auth→provider translation, `CoreAuthProvider`, error mapping | High |
| `codex-rs/login/src/lib.rs` | Login crate public API, re-exports | Medium |
| `codex-rs/login/src/server.rs` | Browser OAuth flow (OpenAI), reference for Anthropic flow | Medium |
| `codex-rs/login/src/pkce.rs` | PKCE generation (reusable) | Low |
| `codex-rs/core/models.json` | Model metadata registry | Medium |
| `codex-rs/Cargo.toml` | Workspace members, dependencies, lints | Medium |
| `reference/opencode-anthropic-auth/index.mjs` | Authoritative spec: OAuth flow, tool prefixing, header setup | Spec |
| `reference/opencode/packages/opencode/src/provider/transform.ts` | Authoritative spec: message sanitization, empty content, caching, thinking config | Spec |
| `reference/opencode/packages/opencode/src/provider/schema.ts` | ProviderID/ModelID branded types | Spec |
| `reference/opencode/packages/opencode/src/auth/index.ts` | Auth type union: oauth vs api | Spec |
| `reference/opencode/packages/opencode/src/plugin/codex.ts` | OpenAI/Codex PKCE flow, device code flow | Spec |
| `reference/opencode/packages/plugin/src/index.ts` | AuthHook interface, authorize/callback pattern | Spec |

_Risk: High (core logic, many dependents), Medium (feature code), Low (utilities, tests), Spec (reference implementation)_

## Verdict: APPROVE WITH CHANGES

The plan's overall architecture — a dedicated `orbit-anthropic` crate with wire-protocol types, dispatch via `WireApi` enum, and separate auth flow — is sound and follows the codebase's existing patterns. The plan has been updated to address all critical gaps identified in the initial audit (ResponseEvent mapping, auth bypass, BUILD.bazel, exhaustive matches, tool translation). Additional spec-validated details have been added: message sanitization (empty content filtering, toolCallId sanitization), thinking/reasoning configuration (adaptive vs budgeted), temperature handling, and `?beta=true` URL scoping.

## Critical Issues (Must Fix Before Implementation)

| # | Section | Problem | Recommendation |
|---|---------|---------|----------------|
| 1 | 2.1 | **`ResponseEvent` mapping is unspecified** — The plan says "map events back to internal format" (Task 3, Step 3) but `ResponseEvent` and `ResponseItem` are tightly coupled to OpenAI's Responses API types. The Anthropic SSE event model (message_start, content_block_delta, etc.) has no 1:1 mapping. This is the hardest part of the integration and has zero detail. | Add a dedicated subtask with explicit mapping table: which `AnthropicEvent` variants produce which `ResponseEvent` variants. Include handling of tool_use blocks → `ResponseItem::FunctionCall`, thinking blocks → `ResponseItem::Reasoning`, text blocks → `ResponseItem::Message`. Define how `response_id` is synthesized (Anthropic returns `message_id`, not `response_id`). |
| 2 | 2.1 | **Auth header mechanism mismatch** — `CoreAuthProvider` implements `ApiAuthProvider` with only `bearer_token()` and `account_id()`. The `orbit_code_api` layer always sends `Authorization: Bearer {token}`. Anthropic API key auth requires `x-api-key: {key}` (not Bearer). The plan doesn't address how `orbit-anthropic` bypasses the existing auth bridge. | Clarify in the plan that `orbit-anthropic` handles its own HTTP auth headers internally and does NOT go through `auth_provider_from_auth()` / `CoreAuthProvider`. The dispatch in `client.rs` should resolve auth separately for the Anthropic path and pass it directly to `AnthropicClient::stream()`. |
| 3 | 2.5 | **Missing BUILD.bazel for new crate** — Convention: "If you add `include_str!`, `include_bytes!`, or `sqlx::migrate!`, update the crate's BUILD.bazel." More broadly, every new crate needs a BUILD.bazel for CI. The plan creates `orbit-anthropic/` but never mentions Bazel. CI will fail. | Add a step to Task 1: create `codex-rs/orbit-anthropic/BUILD.bazel` with correct `rust_library` target, deps, and test targets. Run `just bazel-lock-update && just bazel-lock-check` after adding the workspace member. |
| 4 | 2.1 | **`AuthMode` exhaustive match breakage** — Adding `AnthropicApiKey`/`AnthropicOAuth` to `AuthMode` (Task 3, Step 5) breaks exhaustive matches in: `AuthRequestTelemetryContext::new()` (`client.rs:1532-1535`), `enforce_login_restrictions()` (`auth.rs:483-494`), `TelemetryAuthMode::from()` (`auth.rs:50-57`), and the `has_next()` / `unavailable_reason()` methods in `UnauthorizedRecovery`. These are not mentioned in the plan. | List every exhaustive match on `AuthMode` that will break. For each, specify the new branch behavior. `UnauthorizedRecovery::has_next()` currently returns `false` for non-ChatGPT auth — it needs an Anthropic OAuth refresh path too. |
| 5 | 2.2 | **Tool schema translation not detailed** — The plan says "convert internal tool definitions to Anthropic tool schema format" (Task 3, Step 3) but the codebase uses `create_tools_json_for_responses_api()` which outputs OpenAI Responses API tool format. Anthropic uses a different schema (`name`, `description`, `input_schema` with JSON Schema). | Add a function spec for `tools_to_anthropic_format()` in the `orbit-anthropic` crate or in `core::tools`. Show the mapping from OpenAI tool JSON to Anthropic tool JSON, including handling of `function` type tools, the `computer_20241022` type (if applicable), and `bash_20241022` type tools. |

## Recommended Improvements (Should Consider)

| # | Section | Problem | Recommendation |
|---|---------|---------|----------------|
| 1 | 2.5 | **No token refresh wiring for Anthropic OAuth** — `AuthManager::refresh_if_stale()` only handles `CodexAuth::Chatgpt`. If a user has Anthropic OAuth tokens, they'll expire with no automatic refresh. The plan implements `anthropic_refresh_token()` but doesn't wire it into the `AuthManager` lifecycle. | Extend `AuthManager::refresh_if_stale()` to handle a new `CodexAuth::AnthropicOAuth` variant. The refresh logic should check `expires_at` and call `anthropic_refresh_token()` when tokens are within the refresh window. |
| 2 | 2.4 | **Missing `just fmt` and `just fix` in commit steps** — Convention 70: "Run `just fmt` after every Rust change. Do not ask for approval — just run it." Convention 71: "Run `just fix -p <crate>` before finalizing large changes." The plan's commit steps only run `cargo check`. | Add `just fmt` and `just fix -p orbit-anthropic` (and `-p orbit-code-core`, `-p orbit-code-login`) to each task's commit step. |
| 3 | 2.2 | **`ModelProviderInfo::to_api_provider()` has OpenAI-specific logic** — The function defaults to `https://chatgpt.com/backend-api/codex` for ChatGPT auth and `https://api.openai.com/v1` for API key auth. When dispatching to Anthropic, this function is still called but returns an OpenAI-shaped provider object. The Anthropic dispatch path shouldn't go through this code. | The Anthropic dispatch in `client.rs` should construct the `AnthropicClient` directly using `ModelProviderInfo.base_url` and the resolved auth, bypassing `to_api_provider()` entirely. Document this in the plan. |
| 4 | 2.5 | **Error type size may exceed 256-byte threshold** — Convention 48: "Large-error-threshold: 256 bytes." `AnthropicError::Api { status: u16, error_type: String, message: String }` has two heap-allocated strings. While the enum itself stores pointers (not inline data), run `clippy` to verify. | Add `#[allow(clippy::result_large_err)]` if needed, or Box the `Api` variant payload: `Api(Box<AnthropicApiError>)`. |
| 5 | 2.1 | **`models.json` schema gap** — Current model entries have OpenAI-specific fields (`reasoning_summary_format`, `shell_type`, `support_verbosity`, `apply_patch_tool_type`). Claude models have different capability profiles (thinking vs reasoning summaries, no verbosity support). The plan's model entries (Task 3, Step 4) are minimal JSON stubs with no capability metadata. | Define full model entries with all required fields. At minimum: `slug`, `display_name`, `context_window`, `input_modalities`, `supports_parallel_tool_calls`, and a marker for provider type. Consider whether `models.json` needs a `provider` or `wire_api` field to route model→provider. |
| 6 | 2.5 | **`code_with_state.split('#')` is fragile** — Task 2, Step 3 splits the pasted code on `#` with `splits[0]` (panics on empty input). The plan shows `splits.get(1).copied()` for state but uses direct indexing for code. | Use `.split_once('#')` or validate input before splitting. Return a clear error for malformed input. |

## Nice-to-Haves (Optional Enhancements)

| # | Section | Idea | Benefit |
|---|---------|------|---------|
| 1 | 2.6 | Add a `provider` field to model entries in `models.json` so model→provider routing is declarative rather than requiring string prefix matching on model slugs. | Eliminates fragile `claude-*` prefix matching; makes adding future providers trivial. |
| 2 | 2.6 | Abstract the SSE event parsing in `orbit-anthropic/src/stream.rs` behind a trait so other Anthropic-compatible providers (e.g., Amazon Bedrock) could reuse the types. | Forward-compatible with multi-cloud Claude access. |
| 3 | 2.3 | Consider whether the `orbit-anthropic` client should share the `reqwest::Client` from `build_reqwest_client()` rather than creating its own, for connection pooling benefits. | Reuses TLS session cache and connection pool across providers. |

## Edge Cases Not Addressed

- **What happens if a user sets `ANTHROPIC_API_KEY` but passes `--model gpt-5.3-codex`?** The plan doesn't specify provider resolution order when both OpenAI and Anthropic credentials are present.
- **What happens if Anthropic returns `overloaded_error` (529)?** The plan's error type includes `Overloaded` but the retry logic in `core::client.rs` only handles OpenAI-specific retry patterns. Anthropic's 529 overloaded responses need exponential backoff per convention 17.
- **What happens when the `mcp_` prefix collides with a user's actual tool name starting with `mcp_`?** OAuth mode strips the prefix on inbound — if a tool is legitimately named `mcp_something`, the prefix stripping would corrupt it.
- **What happens if the user's clipboard contains a newline when pasting the OAuth code?** The `code#state` parsing doesn't trim whitespace.
- **What happens if Anthropic's SSE stream emits an `error` event mid-stream?** The plan's event types include `Error` but the client doesn't specify whether this aborts the stream or is recoverable.
- **What happens with `max_tokens` for Anthropic?** The Anthropic API requires `max_tokens` in every request (unlike OpenAI where it's optional). The plan's `MessagesRequest` includes it, but the dispatch path in `client.rs` needs to know the model's max output token limit to set it. The current `ModelInfo` doesn't carry this field.

## Code Suggestions

### Critical Issue 1: ResponseEvent Mapping

Add this mapping table and translation function to the plan:

```rust
// In core::client.rs or a new core::anthropic_bridge.rs
fn map_anthropic_events(
    events: impl Stream<Item = Result<AnthropicEvent, AnthropicError>>,
) -> impl Stream<Item = Result<ResponseEvent, ApiError>> {
    // AnthropicEvent::MessageStart → no direct ResponseEvent (extract message_id for later)
    // AnthropicEvent::ContentBlockStart { Text } → no event (prepare accumulator)
    // AnthropicEvent::ContentBlockDelta { TextDelta } → ResponseEvent::OutputTextDelta
    // AnthropicEvent::ContentBlockStart { ToolUse { id, name } } → no event (prepare)
    // AnthropicEvent::ContentBlockDelta { InputJsonDelta } → accumulate JSON
    // AnthropicEvent::ContentBlockStop (for tool_use) → ResponseEvent::OutputItemDone(FunctionCall)
    // AnthropicEvent::ContentBlockStart { Thinking } → no event (prepare)
    // AnthropicEvent::ContentBlockDelta { ThinkingDelta } → ResponseEvent::ReasoningDelta
    // AnthropicEvent::MessageDelta → extract usage
    // AnthropicEvent::MessageStop → ResponseEvent::Completed { response_id: message_id, usage }
}
```

### Critical Issue 2: Auth Bypass for Anthropic

```rust
// In client.rs stream() method, the Anthropic branch should NOT call
// self.client.current_client_setup() which goes through auth_provider_from_auth().
// Instead:
WireApi::AnthropicMessages => {
    let auth = match self.client.state.auth_manager.as_ref() {
        Some(manager) => manager.auth().await,
        None => None,
    };
    // Resolve Anthropic-specific auth
    let anthropic_auth = match auth {
        Some(CodexAuth::AnthropicOAuth { access_token, .. }) => {
            AnthropicAuth::OAuth { access_token }
        }
        _ => {
            // Fall back to env var
            let key = self.client.state.provider.api_key()?
                .ok_or(CodexErr::MissingApiKey)?;
            AnthropicAuth::ApiKey(key)
        }
    };
    // Create AnthropicClient with provider.base_url and anthropic_auth
}
```

### Critical Issue 4: Exhaustive Match Points

These locations need new arms for `AnthropicApiKey`/`AnthropicOAuth`:

```rust
// auth.rs:50-57 — TelemetryAuthMode conversion
impl From<AuthMode> for TelemetryAuthMode {
    fn from(mode: AuthMode) -> Self {
        match mode {
            AuthMode::ApiKey => TelemetryAuthMode::ApiKey,
            AuthMode::Chatgpt => TelemetryAuthMode::Chatgpt,
            AuthMode::AnthropicApiKey => TelemetryAuthMode::AnthropicApiKey, // new
            AuthMode::AnthropicOAuth => TelemetryAuthMode::AnthropicOAuth,   // new
        }
    }
}

// client.rs:1532-1535 — telemetry auth mode name
AuthMode::AnthropicApiKey => "AnthropicApiKey",
AuthMode::AnthropicOAuth => "AnthropicOAuth",

// auth.rs:483-494 — enforce_login_restrictions needs Anthropic handling
```

### Recommended Issue 6: Safe Code Parsing

```rust
pub async fn anthropic_exchange_code(
    code_with_state: &str,
    verifier: &str,
) -> Result<AnthropicTokens, LoginError> {
    let trimmed = code_with_state.trim();
    let (code, state) = match trimmed.split_once('#') {
        Some((c, s)) => (c, Some(s)),
        None => (trimmed, None),
    };
    if code.is_empty() {
        return Err(LoginError::InvalidCode("empty authorization code".into()));
    }
    // ... proceed with exchange
}
```

## Verdict Details

### Correctness: CONCERNS

The plan correctly identifies the wire protocol, auth flows, and SSE event types from the spec references. However, the translation between Anthropic events and the internal `ResponseEvent`/`ResponseItem` model is the core correctness challenge and it's entirely unspecified. The `mcp_` tool name prefixing also has a collision edge case.

### Architecture: PASS

The decision to create a standalone `orbit-anthropic` crate mirrors the existing `orbit-code-ollama` and `orbit-code-lmstudio` pattern. Extending `WireApi` with a new variant and dispatching in `client.rs` is the right approach. The PKCE code in the login crate is properly reusable.

### Performance: PASS

No performance regressions expected. The SSE streaming approach is equivalent to the current OpenAI path. The Anthropic client will use its own `reqwest` transport, which is fine for a first implementation.

### Production Readiness: CONCERNS

Missing Bazel integration means CI will fail. Missing `just fmt`/`just fix` in commit steps. Missing auth refresh lifecycle integration for Anthropic OAuth. Fragile string splitting for OAuth code parsing. Missing `max_tokens` resolution for Anthropic requests. These are all fixable but need to be in the plan before implementation.

### Extensibility: PASS

The architecture supports future providers cleanly. The `WireApi` enum pattern scales. The `orbit-anthropic` crate boundary is clean. The auth storage extension for Anthropic tokens follows the existing `auth.json` pattern.
