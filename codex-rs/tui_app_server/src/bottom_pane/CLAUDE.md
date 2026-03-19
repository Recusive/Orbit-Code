# codex-rs/tui_app_server/src/bottom_pane/

The interactive footer pane of the chat UI -- prompt input, approval overlays, popups, and modals.

## What this folder does

Implements the `BottomPane` widget that sits at the bottom of the chat interface. It owns the `ChatComposer` (editable prompt input) and a stack of transient `BottomPaneView`s that temporarily replace the composer for focused interactions such as approval dialogs, file search popups, command palettes, selection lists, skill toggles, and user-input forms.

Input routing is layered: `BottomPane` decides which local surface receives a key event (view stack vs. composer), while higher-level intent like "interrupt" or "quit" is decided by the parent `ChatWidget`.

## What it plugs into

- **../chatwidget.rs**: `ChatWidget` owns the `BottomPane` and delegates key events and rendering to it.
- **../app.rs**: `App` pushes views onto the bottom pane stack in response to app-server requests (approvals, user input, MCP elicitations).
- **codex-app-server-protocol**: Approval and elicitation request/response types.

## Key files

| File | Role |
|------|------|
| `mod.rs` | `BottomPane` struct -- manages the composer + view stack, input routing, status line, and transient hints. |
| `bottom_pane_view.rs` | `BottomPaneView` enum -- all possible overlay views the bottom pane can display. |
| `chat_composer.rs` | `ChatComposer` -- the multi-line text input field with paste heuristics, auto-submit, and history. |
| `chat_composer_history.rs` | Input history navigation for the composer (up/down arrow recall). |
| `approval_overlay.rs` | `ApprovalOverlay` -- renders exec command and file change approval dialogs. |
| `mcp_server_elicitation.rs` | `McpServerElicitationOverlay` -- MCP server elicitation form rendering. |
| `request_user_input/` | Subdirectory implementing the multi-question user-input overlay. |
| `file_search_popup.rs` | File search popup with fuzzy matching. |
| `command_popup.rs` | Slash-command palette popup. |
| `skill_popup.rs` | Skill selection popup. |
| `skills_toggle_view.rs` | Toggle view for enabling/disabling skills. |
| `list_selection_view.rs` | Generic list-based selection view. |
| `selection_popup_common.rs` | Shared logic for selection popups (row measurement, display). |
| `multi_select_picker.rs` | Multi-select checkbox picker. |
| `feedback_view.rs` | Feedback submission view. |
| `experimental_features_view.rs` | Experimental feature toggle view. |
| `custom_prompt_view.rs` | Custom system prompt editor view. |
| `app_link_view.rs` | App-link elicitation and suggestion views. |
| `footer.rs` | Footer bar rendering (key hints, status). |
| `unified_exec_footer.rs` | Unified footer for exec-mode display. |
| `status_line_setup.rs` | Status line configuration and formatting. |
| `textarea.rs` | Low-level textarea widget. |
| `scroll_state.rs` | Scroll position tracking for scrollable views. |
| `popup_consts.rs` | Shared popup layout constants. |
| `prompt_args.rs` | Prompt argument parsing for the composer. |
| `slash_commands.rs` | Slash command registry and matching. |
| `paste_burst.rs` | Paste burst detection heuristic. |
| `pending_input_preview.rs` | Preview of pending input during streaming. |
| `pending_thread_approvals.rs` | Tracks pending thread-level approvals. |

## Imports from

- `crate::app_event` / `crate::app_event_sender` -- event types and sender channel.
- `crate::render::renderable` -- `Renderable`, `FlexRenderable`, `RenderableItem` traits.
- `crate::key_hint` -- `KeyBinding` for footer hints.
- `crate::tui::FrameRequester` -- for scheduling redraws.
- `codex_core::features`, `codex_core::plugins`, `codex_core::skills` -- feature flags, plugin capabilities, skill metadata.
- `codex_file_search::FileMatch` -- file search results.
- `codex_protocol::request_user_input` -- user input event types.

## Exports to

- **crate::chatwidget** / **crate::app**: Public types like `ApprovalRequest`, `McpServerElicitationFormRequest`, `RequestUserInputOverlay`, `SelectionItem`, `SelectionViewParams`, `FeedbackAudience`, `AppLinkView`, `ChatComposer`, `InputResult`.
