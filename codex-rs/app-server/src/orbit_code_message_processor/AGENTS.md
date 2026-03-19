# codex-rs/app-server/src/codex_message_processor/

This file applies to `codex-rs/app-server/src/codex_message_processor/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-app-server` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-app-server`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### Purpose

Contains helper submodules for the `CodexMessageProcessor`, which is the core domain request handler in the app-server. These helpers extract and organize supporting logic that would otherwise bloat the main `codex_message_processor.rs` file.

### Key Files

| File | Role |
|------|------|
| `apps_list_helpers.rs` | Helpers for the `apps/list` endpoint. Merges all-connectors and accessible-connectors lists from the ChatGPT connector API, and determines whether an `AppListUpdated` notification should be broadcast. |
| `plugin_app_helpers.rs` | Helpers for loading plugin-related app metadata. Fetches `AppSummary` entries for plugin-associated connector IDs by querying the ChatGPT connector listing API, with a fallback to cached data. |

### What It Plugs Into

- These modules are imported by the parent `codex_message_processor.rs` via `mod codex_message_processor;` submodule paths.
- They support the `apps/list` and `plugin/read` request handlers in `CodexMessageProcessor`.

### Imports From

- `codex-app-server-protocol` -- `AppInfo`, `AppsListResponse`, `AppListUpdatedNotification`, `AppSummary`, `ServerNotification`.
- `codex-chatgpt::connectors` -- Connector listing and merging functions.
- `codex-core::config::Config` -- Configuration access.
- `codex-core::plugins::AppConnectorId` -- Plugin-to-connector ID mapping.
- `crate::outgoing_message::OutgoingMessageSender` -- For broadcasting notifications.
- `crate::error_code` -- Error code constants.

### Exports To

- Used only by the parent `codex_message_processor.rs`. Functions are `pub(super)`.
