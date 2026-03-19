# app-server/src/codex_message_processor

## Purpose

Contains helper submodules for the `CodexMessageProcessor`, which is the core domain request handler in the app-server. These helpers extract and organize supporting logic that would otherwise bloat the main `codex_message_processor.rs` file.

## Key Files

| File | Role |
|------|------|
| `apps_list_helpers.rs` | Helpers for the `apps/list` endpoint. Merges all-connectors and accessible-connectors lists from the ChatGPT connector API, and determines whether an `AppListUpdated` notification should be broadcast. |
| `plugin_app_helpers.rs` | Helpers for loading plugin-related app metadata. Fetches `AppSummary` entries for plugin-associated connector IDs by querying the ChatGPT connector listing API, with a fallback to cached data. |

## What It Plugs Into

- These modules are imported by the parent `codex_message_processor.rs` via `mod codex_message_processor;` submodule paths.
- They support the `apps/list` and `plugin/read` request handlers in `CodexMessageProcessor`.

## Imports From

- `codex-app-server-protocol` -- `AppInfo`, `AppsListResponse`, `AppListUpdatedNotification`, `AppSummary`, `ServerNotification`.
- `codex-chatgpt::connectors` -- Connector listing and merging functions.
- `codex-core::config::Config` -- Configuration access.
- `codex-core::plugins::AppConnectorId` -- Plugin-to-connector ID mapping.
- `crate::outgoing_message::OutgoingMessageSender` -- For broadcasting notifications.
- `crate::error_code` -- Error code constants.

## Exports To

- Used only by the parent `codex_message_processor.rs`. Functions are `pub(super)`.
