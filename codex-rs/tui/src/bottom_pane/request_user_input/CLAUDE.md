# codex-rs/tui/src/bottom_pane/request_user_input/

Request-user-input overlay state machine for multi-question interactive forms.

## What this folder does

Implements the overlay that appears when the agent requests structured user input (via `RequestUserInputEvent`). Handles multi-question flows where each question can be answered by selecting one option and/or providing freeform notes. The overlay manages navigation between questions, input focus switching, and submission of collected answers.

## What it plugs into

- **../mod.rs**: The `BottomPane` pushes a `RequestUserInputOverlay` onto its view stack when a `RequestUserInputEvent` arrives.
- **../../app.rs**: `App` processes the submitted `RequestUserInputResponse` and sends it back to the agent via `Op`.
- **codex-protocol**: Uses `RequestUserInputEvent`, `RequestUserInputAnswer`, `RequestUserInputResponse`, and `TextElement` types.

## Key files

| File | Role |
|------|------|
| `mod.rs` | `RequestUserInputOverlay` -- the main state machine. Manages question queue, selected option per question, notes composer, focus mode (options vs. notes), and submission logic. |
| `layout.rs` | Layout computation for the overlay (option rows, notes area, footer). |
| `render.rs` | Rendering logic -- draws the question title, option list, notes composer, and navigation footer. |

## Behavior

- Each question can have selectable options and/or a freeform notes field.
- Typing while focused on options automatically jumps into the notes area.
- Enter advances to the next question; on the last question it submits all answers.
- Freeform-only questions submit an empty answer list when the notes field is empty.
- Notes are stored per-question and appended as extra answers.

## Sub-directories

| Directory | Purpose |
|-----------|---------|
| `snapshots/` | Insta test snapshots for request_user_input rendering tests. |
