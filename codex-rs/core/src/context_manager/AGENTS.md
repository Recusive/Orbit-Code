# codex-rs/core/src/context_manager/

This file applies to `codex-rs/core/src/context_manager/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Conversation history management, token accounting, and context normalization.

### What this folder does

The `ContextManager` is the in-memory transcript of a thread's conversation history. It tracks all `ResponseItem`s (user messages, assistant messages, tool calls, tool outputs) and maintains token usage metadata for context window management.

Key responsibilities:
- Stores the ordered list of conversation items (oldest first)
- Tracks token usage info from API responses for context window estimation
- Maintains a reference context item (baseline) for efficient model-visible settings diffing between turns
- Provides helpers to estimate model-visible byte sizes of response items
- Supports history replacement during compaction
- Normalizes response items for consistent representation

### Key files

| File | Purpose |
|------|---------|
| `mod.rs` | Module declarations, re-exports `ContextManager`, `TotalTokenUsageBreakdown` |
| `history.rs` | `ContextManager` struct: item storage, token tracking, history queries, byte estimation |
| `history_tests.rs` | Tests for history management |
| `normalize.rs` | Response item normalization (consistent formatting, deduplication) |
| `updates.rs` | History update operations (append, replace, compaction support) |

### Imports from

- `codex_protocol` -- `ResponseItem`, `ContentItem`, `TokenUsage`, `TokenUsageInfo`, `TurnContextItem`
- `crate::truncate` -- Token estimation and truncation utilities
- `crate::event_mapping` -- Contextual user message detection

### Exports to

- `crate::codex::Session` -- `ContextManager` is held in `SessionState` and used for all history operations
- `crate::compact` -- reads history for compaction decisions
- `crate::tasks` -- queries history during turn execution
