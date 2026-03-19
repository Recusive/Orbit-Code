# app-server/tests/common

## Purpose

Shared test support library (`app_test_support` crate) used by the app-server integration test suite. Provides reusable infrastructure for spawning mock servers, generating auth fixtures, writing test configurations, and building mock API responses.

## Key Files

| File | Role |
|------|------|
| `lib.rs` | Crate root. Re-exports all submodule helpers and provides `to_response<T>` for deserializing `JSONRPCResponse` into typed response structs. |
| `mock_model_server.rs` | Creates wiremock-based mock OpenAI API servers that return configurable assistant responses. Supports repeating single responses or sequenced response lists. |
| `auth_fixtures.rs` | `ChatGptAuthFixture` for generating test auth credentials. Includes JWT token encoding (`encode_id_token`) and `write_chatgpt_auth` for writing auth state to disk. |
| `config.rs` | `write_mock_responses_config_toml` -- writes a `config.toml` pointing at a mock model server URL for test isolation. |
| `responses.rs` | Builders for constructing SSE response bodies: `create_exec_command_sse_response`, `create_apply_patch_sse_response`, `create_shell_command_sse_response`, `create_request_user_input_sse_response`, `create_request_permissions_sse_response`, `create_final_assistant_message_sse_response`. |
| `analytics_server.rs` | `start_analytics_events_server` -- mock analytics endpoint for testing telemetry event submission. |
| `mcp_process.rs` | `McpProcess` -- helper for spawning and managing the app-server as a child process for stdio-based integration tests. Provides `DEFAULT_CLIENT_NAME`. |
| `models_cache.rs` | `write_models_cache` / `write_models_cache_with_models` -- writes model cache files for tests that need pre-populated model listings. |
| `rollout.rs` | `create_fake_rollout` / `create_fake_rollout_with_source` / `create_fake_rollout_with_text_elements` -- helpers for generating fake rollout/review data. |

## What It Plugs Into

- Consumed by `tests/suite/` modules and `tests/suite/v2/` modules.
- Has its own `Cargo.toml` and `BUILD.bazel` as a workspace member.

## Imports From

- `codex-app-server-protocol` -- JSON-RPC message types.
- `core_test_support` -- Shell formatting and temp path utilities.
- `wiremock` -- HTTP mock server framework.

## Exports To

- All helpers are `pub` and re-exported from `lib.rs` for use by the test suite.
