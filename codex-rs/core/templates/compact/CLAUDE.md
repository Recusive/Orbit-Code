# codex-rs/core/templates/compact/

Context compaction prompt templates.

## What this folder does

Provides the prompt templates used during context compaction -- the process of summarizing conversation history to fit within the model's context window.

## Key files

| File | Purpose |
|------|---------|
| `prompt.md` | System prompt for the compaction agent, instructing it how to summarize conversation history while preserving critical context |
| `summary_prefix.md` | Prefix template for compaction summaries, providing structure for the summary output |

## Where it plugs into

- Loaded via `include_str!()` in `crate::compact`
- Used by `CompactTask` in `crate::tasks::compact` when context window is exceeded
- Also used by `crate::compact_remote` for remote compaction
