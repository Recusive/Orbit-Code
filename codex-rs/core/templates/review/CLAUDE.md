# codex-rs/core/templates/review/

Code review output format templates and history message templates.

## What this folder does

Defines the XML and markdown templates used for the code review workflow, including output format specifications and conversation history messages.

## Key files

| File | Purpose |
|------|---------|
| `exit_success.xml` | XML template for successful review completion output format |
| `exit_interrupted.xml` | XML template for interrupted review output format |
| `history_message_completed.md` | Markdown template for completed review messages in conversation history |
| `history_message_interrupted.md` | Markdown template for interrupted review messages in conversation history |

## Where it plugs into

- Loaded via `include_str!()` in `crate::review_prompts` and `crate::review_format`
- Used by `crate::tasks::review::ReviewTask` during code review execution
- Output templates define the structured format the review agent must use
