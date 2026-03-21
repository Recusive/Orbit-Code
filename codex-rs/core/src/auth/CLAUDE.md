# codex-rs/core/src/auth/

Multi-provider authentication for Codex CLI — supports OpenAI (ChatGPT) and Anthropic (Claude) simultaneously.

## What this folder does

Implements the full auth lifecycle: credential storage, session caching, save/load with provider merging, and 401 recovery. The v2 storage format (`auth.json`) holds credentials for multiple providers side-by-side, so users can switch between GPT and Claude models without re-authenticating.

## Module structure

| File | Lines | Responsibility |
|------|-------|----------------|
| `../auth.rs` | ~550 | Types: `CodexAuth` enum, `AuthMode`, `ChatgptAuth`, `ApiKeyAuth`, env var constants, re-exports |
| `manager.rs` | ~730 | `AuthManager` — single source of truth for session auth. Caches credentials, handles reload, ChatGPT token refresh, provider-filtered lookups via `auth_cached_for_provider()` |
| `persistence.rs` | ~380 | All disk I/O: `save_auth()`, `save_auth_v2()`, `load_auth()`, `login_with_*()`, `logout()`. Both save functions merge into existing storage to preserve other providers |
| `recovery.rs` | ~240 | `UnauthorizedRecovery` state machine — drives 401 recovery through reload → refresh → external refresh steps |
| `storage.rs` | ~630 | Storage backends (`FileAuthStorage`, `KeyringAuthStorage`, `AutoAuthStorage`, `EphemeralAuthStorage`), `AuthDotJsonV2` format, `ProviderName`/`ProviderAuth` enums, v1↔v2 migration |
| `storage_tests.rs` | ~630 | Storage backend tests |

## Data format (v2)

```json
{
  "version": 2,
  "providers": {
    "openai": {
      "type": "chatgpt",
      "tokens": { "id_token": "...", "access_token": "...", "refresh_token": "..." },
      "last_refresh": "2026-03-21T..."
    },
    "anthropic": {
      "type": "anthropic_oauth",
      "access_token": "sk-ant-oat01-...",
      "refresh_token": "sk-ant-ort01-...",
      "expires_at": 1774151408
    }
  }
}
```

Both `save_auth()` and `save_auth_v2()` merge into existing storage — saving OpenAI auth preserves any existing Anthropic entry and vice versa.

## Key types

- **`CodexAuth`** — enum: `ApiKey`, `Chatgpt`, `ChatgptAuthTokens`, `AnthropicApiKey`, `AnthropicOAuth`
- **`AuthManager`** — session-scoped cache with `auth_cached_for_provider(ProviderName)` for provider-filtered lookups
- **`ProviderName`** — `OpenAI` | `Anthropic` (strongly-typed HashMap key)
- **`ProviderAuth`** — per-provider credential variant (5 types across both providers)
- **`UnauthorizedRecovery`** — state machine for 401 handling (reload → refresh → external)

## Related modules

- **`../anthropic_auth/`** — Anthropic-specific auth: OAuth types, token refresh, request modifications (tool prefixing, system prompt)
- **`../../auth_tests.rs`** — Unit tests for auth loading, provider preservation, manager behavior
- **`../../auth_env_telemetry.rs`** — Tracks which auth env vars are present at startup
