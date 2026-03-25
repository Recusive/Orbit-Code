# Learning 1: Anthropic OAuth Integration & Multi-Provider Auth

> Session date: 2026-03-21  
> Duration: ~8 hours of debugging, implementing, and testing

---

## What Was Built

### Anthropic OAuth Code-Paste Flow

- Full browser-based OAuth flow for Claude Pro/Max subscription users
- TUI onboarding sub-menu: "Sign in with Claude" → API Key or OAuth
- Token exchange, persistence in v2 auth storage, and proactive refresh
- State validation (StaleAttempt error for cross-session codes)

### Files Changed (Auth-Related)


| File                                             | What Changed                                                                                                                                                                                                                  |
| ------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `login/src/anthropic.rs`                         | CA-aware HTTP client, `StaleAttempt` error, state validation, `Result` return for `anthropic_authorize_url`, two authorize URLs (claude.ai vs console.anthropic.com)                                                          |
| `anthropic/src/token_refresh.rs`                 | **New** — `refresh_anthropic_token()` for proactive/forced refresh                                                                                                                                                            |
| `anthropic/src/client.rs`                        | JoinHandle abort-on-drop for SSE stream, OAuth beta headers (`effort-2025-11-24`, `context-management-2025-06-27`), user-agent `claude-cli/2.1.81`                                                                            |
| `core/src/auth.rs`                               | `load_auth()` now loads Anthropic from v2 storage, auth priority (storage before env var), `refresh_anthropic_oauth_if_needed()`, `force_refresh_anthropic_oauth()`, `AnthropicOAuth` recovery mode in `UnauthorizedRecovery` |
| `core/src/auth/anthropic.rs`                     | `AnthropicOAuthAuth` struct with `is_expiring_within()`                                                                                                                                                                       |
| `core/src/client.rs`                             | Proactive OAuth refresh before requests, system prompt prefix for OAuth ("You are Claude Code..."), provider override for mid-session GPT switching, `current_client_setup_with_provider()`                                   |
| `core/src/anthropic_bridge.rs`                   | Adaptive thinking for opus-4-6/sonnet-4-6, 1M context beta only for opus, `uses_adaptive_thinking()` function                                                                                                                 |
| `core/src/default_client.rs`                     | `connect_timeout(10s)` on reqwest client                                                                                                                                                                                      |
| `tui/src/onboarding/auth.rs`                     | `AnthropicPickMethod`, `AnthropicOAuthCodeEntry`, `AnthropicOAuthSuccess` states, OAuth code entry UI with error feedback                                                                                                     |
| `tui_app_server/src/onboarding/auth.rs`          | Mirror of TUI changes                                                                                                                                                                                                         |
| `tui/src/onboarding/onboarding_screen.rs`        | Quit suppression for OAuth code entry, ChatGPT login no longer disabled for Anthropic provider                                                                                                                                |
| `tui/src/status/`                                | `/status` shows "Claude Pro/Max (OAuth)" or "Anthropic API key configured"                                                                                                                                                    |
| `app-server/src/orbit_code_message_processor.rs` | `account/get` handler checks Anthropic auth, `current_account_updated_notification` fallback                                                                                                                                  |


---

## Key Learnings

### 1. Anthropic OAuth Has TWO Authorize Endpoints


| Mode             | URL                                             | Use Case                                         |
| ---------------- | ----------------------------------------------- | ------------------------------------------------ |
| Max Subscription | `https://claude.ai/oauth/authorize`             | Tokens used directly for inference (zero cost)   |
| Console API Key  | `https://console.anthropic.com/oauth/authorize` | Exchange tokens for permanent `sk-ant-`* API key |


Both use the same token endpoint: `https://console.anthropic.com/v1/oauth/token`

Reference implementation: `/reference/opencode-anthropic-auth/index.mjs`

### 2. OAuth Token Exchange Requires `state` in Body

The exchange POST body MUST include the `state` field (the pasted state from the code):

```json
{
    "grant_type": "authorization_code",
    "client_id": "9d1c250a-e61b-44d9-88ed-5944d1962f5e",
    "code": "<code>",
    "state": "<state>",
    "redirect_uri": "https://console.anthropic.com/oauth/code/callback",
    "code_verifier": "<verifier>"
}
```

Without `state`, the API returns `400 Bad Request: Invalid request format`.

### 3. OAuth System Prompt Requirement for Premium Models

The Anthropic OAuth endpoint **requires** the system prompt to start with a separate block:

```json
"system": [
    {"type": "text", "text": "You are Claude Code, Anthropic's official CLI for Claude."},
    {"type": "text", "text": "<actual system prompt>"}
]
```

**Critical**: This MUST be a separate first system block, NOT concatenated into the existing prompt. Concatenation causes `400 invalid_request_error: Error`.

Without this prefix, only `claude-haiku-4-5-20251001` works. With it, `claude-opus-4-6` and `claude-sonnet-4-6` work.

### 4. Required OAuth Beta Headers

Claude Code sends these beta headers for OAuth:

```
anthropic-beta: claude-code-20250219,oauth-2025-04-20,interleaved-thinking-2025-05-14,context-management-2025-06-27,prompt-caching-scope-2026-01-05,effort-2025-11-24
```

Key betas:

- `oauth-2025-04-20` — **Required** for OAuth bearer token auth
- `effort-2025-11-24` — Required for `output_config.effort` parameter
- `context-management-2025-06-27` — Required for context management features

### 5. Adaptive Thinking for Opus 4.6 / Sonnet 4.6

Per Anthropic docs, `thinking: {type: "enabled", budget_tokens: N}` is **deprecated** on Opus 4.6 and Sonnet 4.6. Use:

```json
"thinking": {"type": "adaptive"},
"output_config": {"effort": "high"}
```

### 6. 1M Context Beta — Opus Only via OAuth

- `claude-opus-4-6` supports `context-1m-2025-08-07` beta via OAuth
- `claude-sonnet-4-6` does NOT — returns `429: Extra usage is required for long context requests`
- This is an account-tier limitation on Pro/Max subscriptions

### 7. Tool Name Prefixing for OAuth

When using OAuth, tool names must be prefixed with `mcp_`:

- Request: tool definitions and `tool_use` blocks get `mcp_` prefix
- Response: `mcp_` prefix is stripped from tool names in SSE stream

### 8. OAuth User-Agent

Claude Code uses `claude-cli/2.1.81 (external, cli)` as the user-agent for OAuth requests. We match this.

---

## Auth Storage Architecture

### V2 Format (`~/.orbit/auth.json`)

```json
{
    "version": 2,
    "providers": {
        "anthropic": {
            "type": "anthropic_oauth",
            "access_token": "sk-ant-oat01-...",
            "refresh_token": "sk-ant-ort01-...",
            "expires_at": 1774084058
        },
        "openai": {
            "type": "chatgpt",
            "tokens": { ... },
            "last_refresh": "2026-03-17T..."
        }
    }
}
```

### Auth Loading Priority (`load_auth()`)

1. `OPENAI_API_KEY` / `ORBIT_API_KEY` env var → `ApiKey` auth
2. Ephemeral (in-memory) store → External ChatGPT tokens
3. Persistent storage (file/keyring):
  - If OpenAI provider exists in v2 → Load OpenAI auth
  - If no OpenAI but Anthropic exists → Load Anthropic auth
4. `auth_cached_for_provider()` handles provider-specific lookups with disk fallback

### Home Directory

Resolution: `ORBIT_HOME` > `~/.orbit`

---

## Logging & Debugging

### Log Location

```bash
tail -f ~/.orbit/log/codex-tui.log
```

Log is **truncated on each session start** (not appended) — only shows current session.

### Enabling Debug Logs

```bash
RUST_LOG=orbit_code_core=debug cargo run --bin orbit-code
```

### Debug File Writes (for spawned tasks)

When `tracing::info!` doesn't reach the log file (e.g., from spawned tokio tasks), use:

```rust
let _ = std::fs::write("/tmp/debug-file.txt", format!("value={}", var));
```

### Key Log Lines to Watch

```
load_auth: loaded Anthropic auth from v2 storage    # Auth loaded correctly
resolve_anthropic_auth: found=true auth_mode=Some(AnthropicOAuth)  # OAuth resolved
Anthropic auth resolved is_oauth=true auth_type="bearer_oauth"     # Bearer token used
stream_responses_api: overriding Anthropic provider with OpenAI    # Mid-session GPT switch
Anthropic request details model=... thinking=... beta_headers=...  # Full request params
```

### Intercepting Requests with Proxy

```python
# Simple Python proxy to capture what Claude Code sends
python3 << 'EOF' &
import http.server, json, urllib.request, ssl
class ProxyHandler(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        body = self.rfile.read(int(self.headers.get('Content-Length', 0)))
        with open('/tmp/captured-request.json', 'w') as f:
            json.dump({'headers': dict(self.headers), 'body': json.loads(body)}, f, indent=2)
        # Forward to real API...
server = http.server.HTTPServer(('127.0.0.1', 18080), ProxyHandler)
server.handle_request()
EOF

# Run Claude Code through the proxy
ANTHROPIC_BASE_URL=http://127.0.0.1:18080 claude -p "hi" --model claude-opus-4-6
```

---

## Known Issues — FIXED (Session 2)

### 1. GPT Mid-Session Switching — FIXED

**Status**: Fixed
**Fix**: `current_client_setup_with_provider()` now ALWAYS derives `ProviderName` from `provider.wire_api` and uses `auth_cached_for_provider()` — not just when there's a provider override. `auth_cached_for_provider(OpenAI)` checks env var → v2 storage → returns `None` (never falls through to cached Anthropic auth).

**Additional fix**: `save_auth()` and `save_auth_v2()` merge providers instead of clobbering, so both ChatGPT and Anthropic credentials survive side-by-side in v2 storage.

### 2. Auth Logic — CONSOLIDATED (Session 2)

**Resolved.** Auth logic was consolidated into focused modules:

```
core/src/auth.rs              — Auth types (CodexAuth, AuthMode), constants
core/src/auth/manager.rs      — AuthManager: session cache, provider-filtered lookups
core/src/auth/persistence.rs  — save/load/login/logout (v2 format, merge-on-save)
core/src/auth/recovery.rs     — UnauthorizedRecovery: 401 state machine
core/src/auth/storage.rs      — Storage backends, AuthDotJsonV2, ProviderName
core/src/anthropic_auth/      — Anthropic-specific: types, refresh, request mods
```

Key fix: `save_auth()` and `save_auth_v2()` now merge into existing storage, preserving credentials for other providers. Saving OpenAI auth no longer clobbers Anthropic and vice versa.

### 3. Debug Code — CLEANED UP (Session 2)

**Resolved.** All `std::fs::write("/tmp/...")` debug statements replaced with `tracing::debug!` calls.

---

---

## TMux Testing Pattern

```bash
# Terminal 1: Watch logs
tail -f ~/.orbit/log/codex-tui.log

# Terminal 2: Run the TUI
unset ANTHROPIC_API_KEY  # Force OAuth path
cargo run --bin orbit-code -- --model claude-haiku-4-5-20251001

# Test sequence:
# 1. Send "hello" → should work (Claude OAuth)
# 2. /status → should show "Claude Pro/Max (OAuth)"
# 3. /model claude-opus-4-6 → switch to opus
# 4. Send "hi" → should work (adaptive thinking)
# 5. /model gpt-5.4 → switch to GPT
# 6. Send "hi" → CURRENTLY BROKEN (sends Anthropic token to OpenAI)
```

For intercepting API requests:

```bash
# Check what was sent
cat /tmp/anthropic-request-body.json | python3 -m json.tool | head -20

# Check transport errors
cat /tmp/sse-transport-error.txt
```

