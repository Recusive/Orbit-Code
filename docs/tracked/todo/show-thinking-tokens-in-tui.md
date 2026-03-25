# Plan: Show GPT Thinking Tokens Live in TUI

> **Status:** Todo (reworked after audit 2026-03-24)
> **Audit:** `reviews/show-thinking-tokens-in-tui.audit.md`

## Context

OpenAI's Responses API streams two distinct reasoning event types during GPT model turns:

| API event | Protocol EventMsg | What it is |
|-----------|------------------|------------|
| `response.reasoning_summary_text.delta` | `AgentReasoningDelta` | Model-generated summary of thinking |
| `response.reasoning_text.delta` | `AgentReasoningRawContentDelta` | Raw chain-of-thought (actual thinking tokens) |

The TUI receives both but **displays neither during streaming**. Content is silently accumulated in `reasoning_buffer`, a bold header is extracted for the shimmer status bar, and after completion a `ReasoningSummaryCell` is created (usually transcript-only).

The `exec` CLI already shows raw thinking as plain text with italic magenta styling (`exec/src/event_processor_with_human_output.rs:341-349`).

### Current Event Flow

```
Core (codex.rs:2583) → item.as_legacy_events(show_raw_agent_reasoning)
  ├── Always: AgentReasoningDelta (summary deltas)
  ├── Always: AgentReasoning (summary finalized)
  ├── If flag=true: AgentReasoningRawContentDelta (streaming raw thinking)
  ├── If flag=true: AgentReasoningRawContent (finalized raw thinking block)
  └── If flag=true: AgentReasoningSectionBreak (mid-reasoning boundary)

TUI event handler (chatwidget.rs:5276-5285) — CURRENT (broken):
  AgentReasoningDelta | AgentReasoningRawContentDelta
    → SAME on_agent_reasoning_delta()  ← mixes summary + raw into one buffer
```

### What's Wrong with Cloning PlanStreamController

The original plan proposed cloning `PlanStreamController` (which wraps `StreamState` → `MarkdownStreamCollector`). Three problems:

1. **MarkdownStreamCollector parses markdown.** Raw thinking tokens are plain text. If they contain `**`, backticks, `#`, or `[links]`, the collector misinterprets them as formatting. The exec CLI correctly treats them as plain text (`eprintln!`).

2. **State mixing.** The original plan routed raw deltas through `on_agent_reasoning_delta()` first (for shimmer), then to the thinking controller. This pollutes `reasoning_buffer` with raw text, which then appears in the `ReasoningSummaryCell` transcript. Summary and raw thinking must use **separate state**.

3. **tui_app_server has three paths**, not one. Live notifications (`ServerNotification::ReasoningTextDelta`), replay (`ThreadItem::Reasoning.content`), and test snapshots (`turn_snapshot_events()`) all need handling.

## Approach

Build a standalone **plain-text** `ThinkingStreamController` that does NOT use `StreamState` or `MarkdownStreamCollector`. It owns its own newline-gated text accumulator and FIFO queue, exposing the same `push`/`finalize`/`on_commit_tick` interface so it plugs into the existing commit-tick infrastructure.

**Fully separate summary state from raw-thinking state.** Raw content events never touch `reasoning_buffer`. Summary events never touch the thinking controller. The shimmer header comes exclusively from summary deltas (which is correct — summaries contain the `**bold**` headers).

Gate everything behind the existing `config.show_raw_agent_reasoning` bool (`Config:289`, `ConfigToml:1375`).

## Files to Modify

### 1. `tui/src/streaming/controller.rs` — Add `ThinkingStreamController`

**NOT a clone of PlanStreamController.** Standalone plain-text controller:

```rust
pub(crate) struct ThinkingStreamController {
    /// Accumulates incoming text until newline.
    buffer: String,
    /// FIFO queue of styled, committed lines.
    queue: VecDeque<QueuedLine>,
    /// Whether the "• thinking" header line has been emitted.
    header_emitted: bool,
    /// Set to true after first delta arrives.
    has_seen_delta: bool,
}
```

**Methods (same interface as PlanStreamController):**

| Method | Behavior |
|--------|----------|
| `new()` | Creates empty controller. No `width`/`cwd` params needed (plain text, no markdown link shortening). |
| `push(&mut self, delta: &str) -> bool` | Appends to `buffer`. If newline found: splits on `\n`, styles each line with `.italic().magenta()`, wraps with `"  "` indent via `prefix_lines`, enqueues. Returns `true` if lines were enqueued. |
| `finalize(&mut self) -> Option<Box<dyn HistoryCell>>` | Flushes remaining buffer, drains all queued lines, returns `ThinkingStreamCell`. |
| `on_commit_tick(&mut self) -> (Option<Box<dyn HistoryCell>>, bool)` | Drains one line from queue, wraps in `ThinkingStreamCell`. Returns `(cell, is_idle)`. |
| `on_commit_tick_batch(&mut self, max_lines: usize) -> (Option<Box<dyn HistoryCell>>, bool)` | Drains up to `max_lines`. |
| `queued_lines(&self) -> usize` | Queue depth. |
| `oldest_queued_age(&self, now: Instant) -> Option<Duration>` | Age of front line. |

**Private `emit()` method:**
- Emits `"• ".magenta()` + `"thinking".italic().magenta()` header on first call only
- Wraps content lines with `"  "` indent + `.italic().magenta()` styling
- Returns `ThinkingStreamCell` with `is_stream_continuation` flag

**Plain-text line splitting (replaces MarkdownStreamCollector):**
```rust
fn commit_complete_lines(&mut self) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    while let Some(pos) = self.buffer.find('\n') {
        let line_text = self.buffer[..pos].to_string();
        self.buffer = self.buffer[pos + 1..].to_string();
        lines.push(Line::from(line_text).italic().magenta());
    }
    lines
}
```

### 2. `tui/src/history_cell.rs` — Add `ThinkingStreamCell`

Clone `ProposedPlanStreamCell` (lines 2160-2200):

```rust
pub(crate) struct ThinkingStreamCell {
    lines: Vec<Line<'static>>,
    is_stream_continuation: bool,
}
```

- Factory: `new_thinking_stream(lines, is_stream_continuation) -> ThinkingStreamCell`
- `display_lines()` → returns `self.lines.clone()` (pre-styled by controller — visible in main chat)
- `transcript_lines()` → returns **empty vec** (thinking is display-only, not in Ctrl+T transcript — the summary via `ReasoningSummaryCell` covers the transcript)
- `is_stream_continuation()` → returns stored flag

### 3. `tui/src/streaming/commit_tick.rs` — Add thinking controller param

Extend four functions:

**`run_commit_tick()` (line 69):**
```rust
pub(crate) fn run_commit_tick(
    policy: &mut AdaptiveChunkingPolicy,
    stream_controller: Option<&mut StreamController>,
    plan_stream_controller: Option<&mut PlanStreamController>,
    thinking_stream_controller: Option<&mut ThinkingStreamController>,  // NEW
    scope: CommitTickScope,
    now: Instant,
) -> CommitTickOutput
```

**`stream_queue_snapshot()` (line 97)** — add thinking controller to queue metrics.

**`apply_commit_tick_plan()` (line 148)** — drain order:
1. **Thinking** (first — appears above everything else)
2. **Plan** (second)
3. **Stream/message** (last — agent response text)

**Add `drain_thinking_stream_controller()`** — same shape as `drain_plan_stream_controller()` (line 194).

### 4. `tui/src/chatwidget.rs` — Core integration (SPLIT ROUTING)

**Add field:**
```rust
thinking_stream_controller: Option<ThinkingStreamController>,
```
Initialize as `None` in all 3 constructor paths (~lines 3625, 3813, 3993).

**Split the event match arm** (line 5276). Before:
```rust
EventMsg::AgentReasoningDelta(AgentReasoningDeltaEvent { delta })
| EventMsg::AgentReasoningRawContentDelta(AgentReasoningRawContentDeltaEvent { delta })
    => self.on_agent_reasoning_delta(delta),
```

After:
```rust
EventMsg::AgentReasoningDelta(AgentReasoningDeltaEvent { delta }) => {
    // Summary deltas → shimmer header + transcript buffer ONLY
    self.on_agent_reasoning_delta(delta);
}
EventMsg::AgentReasoningRawContentDelta(AgentReasoningRawContentDeltaEvent { delta }) => {
    // Raw thinking deltas → thinking stream controller ONLY
    // Does NOT touch reasoning_buffer — complete state separation
    self.handle_thinking_delta(delta);
}
```

**Split `AgentReasoningRawContent` handler** (line 5281). Before:
```rust
EventMsg::AgentReasoningRawContent(AgentReasoningRawContentEvent { text }) => {
    self.on_agent_reasoning_delta(text);
    self.on_agent_reasoning_final();
}
```

After:
```rust
EventMsg::AgentReasoningRawContent(AgentReasoningRawContentEvent { text }) => {
    // Deduplication guard: if deltas already streamed live, the controller
    // already has the content — just finalize. Only push the full text when
    // no deltas arrived (non-streaming / fallback path).
    if self.thinking_stream_controller.is_some() {
        self.handle_thinking_finalize();
    } else {
        self.handle_thinking_delta(text);
        self.handle_thinking_finalize();
    }
}
```

**Split section break handler** (line 5285). Before:
```rust
EventMsg::AgentReasoningSectionBreak(_) => self.on_reasoning_section_break(),
```

After:
```rust
EventMsg::AgentReasoningSectionBreak(_) => {
    self.on_reasoning_section_break();       // summary buffer management (unchanged)
    self.handle_thinking_section_break();    // visual separator in thinking stream
}
```

**Add `handle_thinking_delta(&mut self, delta: String)`:**
- Early return if `!self.config.show_raw_agent_reasoning` (defense in depth — core should never send these, but guard anyway)
- Lazy-init `ThinkingStreamController` on first delta
- Call `controller.push(&delta)` → if `true`: send `StartCommitAnimation` + `run_catch_up_commit_tick()`
- Call `request_redraw()`

**Add `handle_thinking_finalize(&mut self)`:**
- If thinking controller exists: `.take()` + `.finalize()` → `add_boxed_history(cell)`

**Add `handle_thinking_section_break(&mut self)`:**
- If thinking controller exists: push `"---"` separator through controller (does NOT finalize)

**Modify `on_agent_reasoning_delta()`** (~line 1658):
- **NO CHANGES.** This function stays summary-only. It writes to `reasoning_buffer`, extracts shimmer headers, and nothing else. Raw deltas never reach it.

**Modify `on_agent_reasoning_final()`** (~line 1679):
- **NO CHANGES to summary handling.** `full_reasoning_buffer` → `ReasoningSummaryCell` → transcript.
- Thinking controller finalization is handled by `handle_thinking_finalize()`, called from the `AgentReasoningRawContent` event handler (NOT from here — summary and raw have separate lifecycle events).

**Modify `on_task_started()`** (~line 1703):
- Add: `self.thinking_stream_controller = None;`

**Modify `on_task_complete()` (~line 1729):**
- Add fallback finalization: if thinking controller still exists (interruption/abort), finalize it.

**Modify `run_commit_tick_with_scope()`** (~line 3063):
- Pass `self.thinking_stream_controller.as_mut()` as the new param.

**Modify `stream_controllers_idle()`** (~line 1123):
- Add thinking controller to idle check.

**Clear/finalize thinking controller at these sites:**

| Site | Line | Action | Reason |
|------|------|--------|--------|
| `on_task_started()` | 1712 | `= None` | New turn, reset |
| `on_task_complete()` | ~1738 | `.take().finalize()` | Fallback for interrupted turns |
| User submit path 1 | 4205 | `= None` | User interrupts with new message |
| User submit path 2 | 4741 | `= None` | User interrupts with slash command |
| `AgentReasoningRawContent` handler | 5281 | `.take().finalize()` | Non-streaming complete block |

**NOT cleared at:**
- `on_agent_reasoning_final()` — that's summary lifecycle, not raw thinking
- `on_reasoning_section_break()` — section break continues thinking stream

**Update trace-log skip list** (line 5247-5251):
- Add `EventMsg::AgentReasoningRawContentDelta(_)` to the high-frequency skip list (it's a streaming delta, no need to trace-log each one)

### 5. Mirror in `tui_app_server/`

Mirror all changes to:
- `tui_app_server/src/streaming/controller.rs` — ThinkingStreamController
- `tui_app_server/src/streaming/commit_tick.rs` — thinking controller param
- `tui_app_server/src/history_cell.rs` — ThinkingStreamCell
- `tui_app_server/src/chatwidget.rs` — field, routing, handlers, clear sites

**PLUS tui_app_server-specific paths:**

**Live notification path** (chatwidget.rs:5877-5880). Before:
```rust
ServerNotification::ReasoningTextDelta(notification) => {
    if self.config.show_raw_agent_reasoning {
        self.on_agent_reasoning_delta(notification.delta);
    }
}
```

After:
```rust
ServerNotification::ReasoningTextDelta(notification) => {
    if self.config.show_raw_agent_reasoning {
        self.handle_thinking_delta(notification.delta);  // thinking stream, NOT summary buffer
    }
}
```

**Section break notification** (chatwidget.rs:5882). Before:
```rust
ServerNotification::ReasoningSummaryPartAdded(_) => self.on_reasoning_section_break(),
```

After:
```rust
ServerNotification::ReasoningSummaryPartAdded(_) => {
    self.on_reasoning_section_break();
    self.handle_thinking_section_break();
}
```

**Replay/resume path** (chatwidget.rs:5551-5563). Before:
```rust
ThreadItem::Reasoning { summary, content, .. } => {
    for delta in summary {
        self.on_agent_reasoning_delta(delta);
    }
    if self.config.show_raw_agent_reasoning {
        for delta in content {
            self.on_agent_reasoning_delta(delta);  // ← WRONG: mixes into summary buffer
        }
    }
    self.on_agent_reasoning_final();
}
```

After:
```rust
ThreadItem::Reasoning { summary, content, .. } => {
    for delta in summary {
        self.on_agent_reasoning_delta(delta);
    }
    if self.config.show_raw_agent_reasoning {
        // Replay: push all content through thinking controller, then immediately finalize.
        // No animation — this is historical data being restored.
        for delta in content {
            self.handle_thinking_delta(delta);
        }
        self.handle_thinking_finalize();
    }
    self.on_agent_reasoning_final();
}
```

**Test snapshot path** (app_server_adapter.rs:860-871):
- `TurnItem::Reasoning` calls `item.as_legacy_events(show_raw_agent_reasoning)` which emits legacy `EventMsg`s. These get handled by the split match arm in `handle_orbit_code_event()`. No changes needed here — the legacy EventMsg routing already does the right thing after the split.

### 6. Tests

**`tui/src/chatwidget/tests.rs`** — add:
- `thinking_streams_when_raw_content_delta_enabled` — set `show_raw_agent_reasoning=true`, feed `AgentReasoningRawContentDelta` events, verify `ThinkingStreamCell`s emitted
- `thinking_not_in_summary_buffer` — feed raw deltas, verify `reasoning_buffer` stays empty (state separation)
- `summary_not_in_thinking_stream` — feed summary deltas, verify no `ThinkingStreamCell` emitted
- `thinking_hidden_when_disabled` — verify no thinking cells when flag is false
- `thinking_finalized_on_raw_content_event` — feed `AgentReasoningRawContent` (complete block, NO prior deltas), verify push + finalization
- `thinking_no_duplication_when_deltas_preceded` — feed `AgentReasoningRawContentDelta` deltas, then `AgentReasoningRawContent` with same text, verify content appears ONCE (dedup guard)
- `thinking_section_break_adds_separator` — verify section break doesn't finalize, adds separator
- `thinking_finalized_on_task_complete` — verify fallback finalization on interrupted turn
- `shimmer_header_unaffected` — verify `**bold**` extraction from summary deltas still works
- `thinking_transcript_lines_empty` — verify `ThinkingStreamCell::transcript_lines()` returns empty (display-only, not in Ctrl+T)

Mirror in `tui_app_server/src/chatwidget/tests.rs` plus:
- `thinking_replay_renders_immediately` — replay `ThreadItem::Reasoning` with content, verify thinking cells in main history
- `thinking_replay_not_in_transcript` — replay thinking, verify Ctrl+T transcript does NOT include thinking cells
- `thinking_live_delta_routes_correctly` — verify `ReasoningTextDelta` notification feeds thinking controller
- `thinking_no_duplication_on_item_completed` — stream raw deltas live, then receive `ItemCompleted` with same content, verify no duplication

**`tui/src/streaming/controller.rs` tests** — add:
- `thinking_controller_push_enqueues_on_newline` — basic lifecycle
- `thinking_controller_no_enqueue_without_newline` — partial line stays buffered
- `thinking_controller_finalize_flushes_partial` — finalize drains remaining buffer
- `thinking_controller_commit_tick_drains_one` — single-line drain
- `thinking_controller_header_emitted_once` — "• thinking" header appears exactly once
- `thinking_controller_plain_text_no_markdown` — text with `**bold**` and backticks renders literally, not as formatting

**Snapshot tests** — accept new snapshots for thinking cell rendering.

## What Does NOT Change

- `protocol/` — all 5 event types already exist
- `core/` — already gates raw event emission on `show_raw_agent_reasoning`
- `core/src/config/mod.rs` — `show_raw_agent_reasoning` field already exists
- `exec/` — already shows raw reasoning correctly
- `app-server-protocol/` — `ReasoningTextDelta`, `ReasoningSummaryTextDelta`, `ThreadItem::Reasoning` already defined
- `ReasoningSummaryCell` — continues to work for transcript-only finalized content (now purely from summary, never polluted by raw thinking)
- `reasoning_buffer` / `full_reasoning_buffer` — unchanged, now cleanly summary-only
- `on_agent_reasoning_delta()` / `on_agent_reasoning_final()` — unchanged, summary-only lifecycle
- No config schema, app-server schema, SDK, Bazel lockfile, or `BUILD.bazel` changes

## Visual Treatment

Thinking text renders as (exec parity):
- Header: `"• ".magenta()` + `"thinking".italic().magenta()` (emitted once per thinking block)
- Content: `.italic().magenta()`, indented with `"  "` prefix
- Section breaks: `"  ---".magenta().dim()` separator line
- Plain text — no markdown interpretation. `**` renders as literal `**`, backticks render as literal backticks.
- ANSI-only palette (magenta is allowed; no Rgb, Indexed, blue, yellow, white, black)

## Edge Cases Addressed

| Edge Case | Handling |
|-----------|----------|
| Turn interruption before reasoning finalization | `on_task_complete()` fallback-finalizes thinking controller |
| Resume/fork/thread-snapshot replay | `ThreadItem::Reasoning.content` → immediate push + finalize (no animation). Cells appear in main history but NOT in Ctrl+T transcript (display-only). |
| `AgentReasoningRawContent` after streamed deltas | **Deduplication guard**: if `thinking_stream_controller.is_some()`, just finalize — don't re-push the full text. Only push when no controller exists (no deltas arrived). |
| Turn with only final block, no streaming deltas | `AgentReasoningRawContent` handler sees no controller → pushes full text → finalizes. Works as non-streaming fallback. |
| `ReasoningSummaryPartAdded` during raw thinking | Routes to both `on_reasoning_section_break()` (summary) and `handle_thinking_section_break()` (thinking stream) |
| Raw thinking containing markdown syntax | Plain-text rendering — no markdown parser, characters render literally |
| `show_raw_agent_reasoning` changes mid-session | Defense-in-depth guard in `handle_thinking_delta()` — core-level gating is primary |
| Partial raw content delivery (network interruption) | `on_task_complete()` finalizes whatever is buffered; partial content renders as-is |
| Ctrl+T transcript with thinking enabled | `ThinkingStreamCell::transcript_lines()` returns empty — transcript shows summary via `ReasoningSummaryCell` only |
| Both summary cell and thinking cells visible | Yes — both appear in main history. Summary cell is usually `transcript_only=true` (display-hidden), thinking cells are display-visible but transcript-hidden. Complementary. |

## Implementation Order

1. **ThinkingStreamCell** in `history_cell.rs` (no dependencies)
2. **ThinkingStreamController** in `controller.rs` (depends on ThinkingStreamCell, standalone plain-text — no StreamState)
3. **commit_tick.rs** changes (depends on ThinkingStreamController)
4. **chatwidget.rs** event routing split + new handlers (depends on all above)
5. **Mirror to tui_app_server/** including app-server-specific paths
6. **Tests** (after all code is in place)
7. **Snapshots** (accept after tests pass)

## Verification

1. **Build**: `cargo build -p orbit-code-tui && cargo build -p orbit-code-tui-app-server`
2. **Clippy**: `just fix -p orbit-code-tui && just fix -p orbit-code-tui-app-server`
3. **Format**: `just fmt`
4. **Tests**: `cargo test -p orbit-code-tui && cargo test -p orbit-code-tui-app-server`
5. **Snapshots**: `cargo insta accept -p orbit-code-tui && cargo insta accept -p orbit-code-tui-app-server`
6. **Manual test (enabled)**: `show-raw-agent-reasoning = true` → send prompt to o3/o4-mini → verify thinking tokens stream live in italic magenta above the agent response
7. **Manual test (disabled)**: No config flag → verify only shimmer status shows (existing behavior preserved)
8. **Manual test (section breaks)**: Long reasoning chain → separator lines render without breaking stream
9. **Manual test (resume)**: Resume a session that had thinking → verify thinking cells appear in history
10. **Manual test (state separation)**: Verify transcript overlay (Ctrl+T) shows summary content only, no raw thinking mixed in
