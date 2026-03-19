# codex-rs/core/templates/compact/

This file applies to `codex-rs/core/templates/compact/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Context compaction prompt templates.

### What this folder does

Provides the prompt templates used during context compaction -- the process of summarizing conversation history to fit within the model's context window.

### Key files

| File | Purpose |
|------|---------|
| `prompt.md` | System prompt for the compaction agent, instructing it how to summarize conversation history while preserving critical context |
| `summary_prefix.md` | Prefix template for compaction summaries, providing structure for the summary output |

### Where it plugs into

- Loaded via `include_str!()` in `crate::compact`
- Used by `CompactTask` in `crate::tasks::compact` when context window is exceeded
- Also used by `crate::compact_remote` for remote compaction
