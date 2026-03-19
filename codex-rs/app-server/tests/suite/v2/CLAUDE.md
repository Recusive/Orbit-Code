# app-server/tests/suite/v2

## Purpose

Comprehensive integration test suite for the v2 app-server protocol. This is the largest test directory, containing per-feature test modules that exercise the full request/response lifecycle through the app-server.

## Structure

Declared in `mod.rs`, each module maps to a specific feature or API endpoint:

### Thread Lifecycle
- `thread_start.rs`, `thread_resume.rs`, `thread_fork.rs`, `thread_read.rs`, `thread_list.rs`, `thread_loaded_list.rs`, `thread_archive.rs`, `thread_unarchive.rs`, `thread_unsubscribe.rs`, `thread_rollback.rs`, `thread_status.rs`, `thread_metadata_update.rs`, `thread_name_websocket.rs`, `thread_shell_command.rs`

### Turn Management
- `turn_start.rs`, `turn_start_zsh_fork.rs`, `turn_interrupt.rs`, `turn_steer.rs`

### Account and Auth
- `account.rs`, `analytics.rs`, `rate_limits.rs`

### Configuration
- `config_rpc.rs`

### Approvals and Permissions
- `request_permissions.rs`, `request_user_input.rs`, `safety_check_downgrade.rs`, `mcp_server_elicitation.rs`

### Plugins
- `plugin_install.rs`, `plugin_list.rs`, `plugin_read.rs`, `plugin_uninstall.rs`

### Other Features
- `app_list.rs`, `collaboration_mode_list.rs`, `command_exec.rs`, `compaction.rs`, `connection_handling_websocket.rs`, `connection_handling_websocket_unix.rs`, `dynamic_tools.rs`, `experimental_api.rs`, `experimental_feature_list.rs`, `fs.rs`, `initialize.rs`, `model_list.rs`, `output_schema.rs`, `plan_item.rs`, `realtime_conversation.rs`, `review.rs`, `skills_list.rs`, `windows_sandbox_setup.rs`

## What It Plugs Into

- Included by `tests/suite/mod.rs` via `mod v2;`.
- Uses `app_test_support` from `tests/common/` for all test infrastructure.

## Imports From

- `app_test_support` -- Mock servers, auth fixtures, process management.
- `codex-app-server-protocol` -- Full set of v2 request/response/notification types.

## Exports To

- No exports; test-only modules.
