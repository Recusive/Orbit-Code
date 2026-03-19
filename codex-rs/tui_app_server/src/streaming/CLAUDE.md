# codex-rs/tui_app_server/src/streaming/

Streaming pipeline for progressive markdown rendering in the chat transcript.

## What this folder does

Implements the streaming infrastructure that turns agent response deltas into progressively rendered chat transcript lines. The pipeline manages newline-gated markdown collection, a FIFO queue of committed render lines, adaptive chunking for smooth display pacing, and orchestration of commit-tick drains across concurrent streams (message and plan).

The key invariant is queue ordering: all drains pop from the front, and each enqueue records an arrival timestamp so policy code can reason about oldest queued age.

## What it plugs into

- **../chatwidget.rs**: `ChatWidget` owns `StreamController` and `PlanStreamController` instances and drives them from agent chunk events.
- **../app.rs**: `App` triggers commit ticks on animation frames and stream completion.
- **../history_cell.rs**: Drained lines are emitted as `HistoryCell` entries.

## Key files

| File | Role |
|------|------|
| `mod.rs` | `StreamState` struct -- holds the `MarkdownStreamCollector` and FIFO `VecDeque<QueuedLine>` of committed lines. Provides `new()`, `clear()`, `step()`, `enqueue()`, and queue introspection. |
| `controller.rs` | `StreamController` and `PlanStreamController` -- manage newline-gated streaming, header emission, and commit animation across message and plan streams. `push()` accepts deltas, `commit_tick()` drains queued lines, `finish()` flushes remaining content. |
| `chunking.rs` | `AdaptiveChunkingPolicy` -- two-gear chunking system (Smooth and CatchUp modes) with hysteresis. Determines how many lines to drain per tick based on queue pressure (depth and age). |
| `commit_tick.rs` | `run_commit_tick()` -- orchestrates commit-tick drains across streaming controllers. Computes queue pressure via `stream_queue_snapshot()`, resolves a chunking plan via `resolve_chunking_plan()`, and applies it via `apply_commit_tick_plan()`. |

## Architecture

```
Agent delta -> StreamController::push()
  -> MarkdownStreamCollector (newline-gated markdown parsing)
  -> committed lines enqueued in StreamState::queued_lines
  -> commit_tick() called on animation frame
       -> AdaptiveChunkingPolicy decides drain count (Smooth: 1 line, CatchUp: all)
       -> drained lines emitted as HistoryCell entries
```

## Imports from

- `crate::markdown_stream::MarkdownStreamCollector` -- incremental markdown parsing.
- `crate::history_cell` -- `HistoryCell` for transcript emission.
- `crate::style` -- styling for plan headers.
- `crate::render::line_utils` -- line prefixing.

## Exports to

- **crate::chatwidget**: `StreamController`, `PlanStreamController`, `AdaptiveChunkingPolicy`, `run_commit_tick`, `CommitTickOutput`.
