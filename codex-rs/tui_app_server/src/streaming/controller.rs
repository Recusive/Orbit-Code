use std::collections::VecDeque;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;

use crate::history_cell::HistoryCell;
use crate::history_cell::{self};
use crate::render::line_utils::prefix_lines;
use crate::style::proposed_plan_style;
use ratatui::prelude::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;

use super::StreamState;

/// Controller that manages newline-gated streaming, header emission, and
/// commit animation across streams.
pub(crate) struct StreamController {
    state: StreamState,
    finishing_after_drain: bool,
    header_emitted: bool,
}

impl StreamController {
    /// Create a controller whose markdown renderer shortens local file links relative to `cwd`.
    ///
    /// The controller snapshots the path into stream state so later commit ticks and finalization
    /// render against the same session cwd that was active when streaming started.
    pub(crate) fn new(width: Option<usize>, cwd: &Path) -> Self {
        Self {
            state: StreamState::new(width, cwd),
            finishing_after_drain: false,
            header_emitted: false,
        }
    }

    /// Push a delta; if it contains a newline, commit completed lines and start animation.
    pub(crate) fn push(&mut self, delta: &str) -> bool {
        let state = &mut self.state;
        if !delta.is_empty() {
            state.has_seen_delta = true;
        }
        state.collector.push_delta(delta);
        if delta.contains('\n') {
            let newly_completed = state.collector.commit_complete_lines();
            if !newly_completed.is_empty() {
                state.enqueue(newly_completed);
                return true;
            }
        }
        false
    }

    /// Finalize the active stream. Drain and emit now.
    pub(crate) fn finalize(&mut self) -> Option<Box<dyn HistoryCell>> {
        // Finalize collector first.
        let remaining = {
            let state = &mut self.state;
            state.collector.finalize_and_drain()
        };
        // Collect all output first to avoid emitting headers when there is no content.
        let mut out_lines = Vec::new();
        {
            let state = &mut self.state;
            if !remaining.is_empty() {
                state.enqueue(remaining);
            }
            let step = state.drain_all();
            out_lines.extend(step);
        }

        // Cleanup
        self.state.clear();
        self.finishing_after_drain = false;
        self.emit(out_lines)
    }

    /// Step animation: commit at most one queued line and handle end-of-drain cleanup.
    pub(crate) fn on_commit_tick(&mut self) -> (Option<Box<dyn HistoryCell>>, bool) {
        let step = self.state.step();
        (self.emit(step), self.state.is_idle())
    }

    /// Step animation: commit at most `max_lines` queued lines.
    ///
    /// This is intended for adaptive catch-up drains. Callers should keep `max_lines` bounded; a
    /// very large value can collapse perceived animation into a single jump.
    pub(crate) fn on_commit_tick_batch(
        &mut self,
        max_lines: usize,
    ) -> (Option<Box<dyn HistoryCell>>, bool) {
        let step = self.state.drain_n(max_lines.max(1));
        (self.emit(step), self.state.is_idle())
    }

    /// Returns the current number of queued lines waiting to be displayed.
    pub(crate) fn queued_lines(&self) -> usize {
        self.state.queued_len()
    }

    /// Returns the age of the oldest queued line.
    pub(crate) fn oldest_queued_age(&self, now: Instant) -> Option<Duration> {
        self.state.oldest_queued_age(now)
    }

    fn emit(&mut self, lines: Vec<Line<'static>>) -> Option<Box<dyn HistoryCell>> {
        if lines.is_empty() {
            return None;
        }
        Some(Box::new(history_cell::AgentMessageCell::new(lines, {
            let header_emitted = self.header_emitted;
            self.header_emitted = true;
            !header_emitted
        })))
    }
}

/// Controller that streams proposed plan markdown into a styled plan block.
pub(crate) struct PlanStreamController {
    state: StreamState,
    header_emitted: bool,
    top_padding_emitted: bool,
}

impl PlanStreamController {
    /// Create a plan-stream controller whose markdown renderer shortens local file links relative
    /// to `cwd`.
    ///
    /// The controller snapshots the path into stream state so later commit ticks and finalization
    /// render against the same session cwd that was active when streaming started.
    pub(crate) fn new(width: Option<usize>, cwd: &Path) -> Self {
        Self {
            state: StreamState::new(width, cwd),
            header_emitted: false,
            top_padding_emitted: false,
        }
    }

    /// Push a delta; if it contains a newline, commit completed lines and start animation.
    pub(crate) fn push(&mut self, delta: &str) -> bool {
        let state = &mut self.state;
        if !delta.is_empty() {
            state.has_seen_delta = true;
        }
        state.collector.push_delta(delta);
        if delta.contains('\n') {
            let newly_completed = state.collector.commit_complete_lines();
            if !newly_completed.is_empty() {
                state.enqueue(newly_completed);
                return true;
            }
        }
        false
    }

    /// Finalize the active stream. Drain and emit now.
    pub(crate) fn finalize(&mut self) -> Option<Box<dyn HistoryCell>> {
        let remaining = {
            let state = &mut self.state;
            state.collector.finalize_and_drain()
        };
        let mut out_lines = Vec::new();
        {
            let state = &mut self.state;
            if !remaining.is_empty() {
                state.enqueue(remaining);
            }
            let step = state.drain_all();
            out_lines.extend(step);
        }

        self.state.clear();
        self.emit(out_lines, /*include_bottom_padding*/ true)
    }

    /// Step animation: commit at most one queued line and handle end-of-drain cleanup.
    pub(crate) fn on_commit_tick(&mut self) -> (Option<Box<dyn HistoryCell>>, bool) {
        let step = self.state.step();
        (
            self.emit(step, /*include_bottom_padding*/ false),
            self.state.is_idle(),
        )
    }

    /// Step animation: commit at most `max_lines` queued lines.
    ///
    /// This is intended for adaptive catch-up drains. Callers should keep `max_lines` bounded; a
    /// very large value can collapse perceived animation into a single jump.
    pub(crate) fn on_commit_tick_batch(
        &mut self,
        max_lines: usize,
    ) -> (Option<Box<dyn HistoryCell>>, bool) {
        let step = self.state.drain_n(max_lines.max(1));
        (
            self.emit(step, /*include_bottom_padding*/ false),
            self.state.is_idle(),
        )
    }

    /// Returns the current number of queued plan lines waiting to be displayed.
    pub(crate) fn queued_lines(&self) -> usize {
        self.state.queued_len()
    }

    /// Returns the age of the oldest queued plan line.
    pub(crate) fn oldest_queued_age(&self, now: Instant) -> Option<Duration> {
        self.state.oldest_queued_age(now)
    }

    fn emit(
        &mut self,
        lines: Vec<Line<'static>>,
        include_bottom_padding: bool,
    ) -> Option<Box<dyn HistoryCell>> {
        if lines.is_empty() && !include_bottom_padding {
            return None;
        }

        let mut out_lines: Vec<Line<'static>> = Vec::new();
        let is_stream_continuation = self.header_emitted;
        if !self.header_emitted {
            out_lines.push(vec!["• ".dim(), "Proposed Plan".bold()].into());
            out_lines.push(Line::from(" "));
            self.header_emitted = true;
        }

        let mut plan_lines: Vec<Line<'static>> = Vec::new();
        if !self.top_padding_emitted {
            plan_lines.push(Line::from(" "));
            self.top_padding_emitted = true;
        }
        plan_lines.extend(lines);
        if include_bottom_padding {
            plan_lines.push(Line::from(" "));
        }

        let plan_style = proposed_plan_style();
        let plan_lines = prefix_lines(plan_lines, "  ".into(), "  ".into())
            .into_iter()
            .map(|line| line.style(plan_style))
            .collect::<Vec<_>>();
        out_lines.extend(plan_lines);

        Some(Box::new(history_cell::new_proposed_plan_stream(
            out_lines,
            is_stream_continuation,
        )))
    }
}

/// Queued line for the thinking stream controller.
///
/// This is separate from [`super::QueuedLine`] because [`ThinkingStreamController`] does not use
/// [`StreamState`] — it manages its own plain-text buffer and queue.
struct ThinkingQueuedLine {
    line: Line<'static>,
    enqueued_at: Instant,
}

/// Controller that streams raw model thinking tokens as plain italic magenta text.
///
/// Unlike [`StreamController`] and [`PlanStreamController`], this controller does **not** use
/// [`StreamState`] or [`MarkdownStreamCollector`]. Raw thinking tokens are plain text —
/// characters like `**`, backticks, and `#` must render literally, not as markdown formatting.
pub(crate) struct ThinkingStreamController {
    /// Accumulates incoming text until a newline is found.
    buffer: String,
    /// FIFO queue of styled, committed lines.
    queue: VecDeque<ThinkingQueuedLine>,
    /// Whether the "Thought:" header line has been emitted.
    header_emitted: bool,
    /// Set to true after the first delta arrives.
    has_seen_delta: bool,
    /// When true, flush every delta immediately (no animation).
    reduced_motion: bool,
}

impl ThinkingStreamController {
    /// Creates an empty thinking stream controller.
    ///
    /// No `width` or `cwd` parameters are needed — thinking tokens are plain text with no
    /// markdown link shortening.
    pub(crate) fn new(reduced_motion: bool) -> Self {
        Self {
            buffer: String::new(),
            queue: VecDeque::new(),
            header_emitted: false,
            has_seen_delta: false,
            reduced_motion,
        }
    }

    /// Push a thinking delta. Lines are enqueued on natural newlines and soft-wrapped
    /// at `SOFT_LINE_WIDTH` for animation. Width-based display wrapping is handled by
    /// `adaptive_wrap_lines` in `ThinkingStreamCell::display_lines`.
    ///
    /// When `reduced_motion` is set, deltas are buffered silently and `finalize()` flushes
    /// everything at once — no animation.
    pub(crate) fn push(&mut self, delta: &str) -> bool {
        const SOFT_LINE_WIDTH: usize = 72;

        if !delta.is_empty() {
            self.has_seen_delta = true;
        }
        self.buffer.push_str(delta);

        // Reduced motion: just buffer — finalize() will flush everything at once.
        if self.reduced_motion {
            return false;
        }

        let mut enqueued = false;

        // Commit lines split on natural newlines.
        let completed = self.commit_complete_lines();
        if !completed.is_empty() {
            let now = Instant::now();
            self.queue
                .extend(completed.into_iter().map(|line| ThinkingQueuedLine {
                    line,
                    enqueued_at: now,
                }));
            enqueued = true;
        }

        // Soft-wrap at word boundaries for animation. Lines are raw text (no prefix) —
        // adaptive_wrap_lines handles all indentation at display time.
        while self.buffer.len() >= SOFT_LINE_WIDTH {
            let split_at = self.buffer[..SOFT_LINE_WIDTH]
                .rfind(' ')
                .unwrap_or(SOFT_LINE_WIDTH);
            let line_text: String = self.buffer.drain(..split_at).collect();
            // Strip the space at the split point so the next line starts cleanly.
            if self.buffer.starts_with(' ') {
                self.buffer.drain(..1);
            }
            self.queue.push_back(ThinkingQueuedLine {
                line: Line::from(Span::from(line_text).italic().magenta()),
                enqueued_at: Instant::now(),
            });
            enqueued = true;
        }

        enqueued
    }

    /// Insert a visual section break separator without finalizing the stream.
    pub(crate) fn push_section_break(&mut self) {
        // Flush any buffered partial line first so the separator appears in order.
        let completed = self.commit_complete_lines();
        if !completed.is_empty() {
            let now = Instant::now();
            self.queue
                .extend(completed.into_iter().map(|line| ThinkingQueuedLine {
                    line,
                    enqueued_at: now,
                }));
        }
        let now = Instant::now();
        self.queue.push_back(ThinkingQueuedLine {
            line: Line::from("---".magenta().dim()),
            enqueued_at: now,
        });
    }

    /// Finalize the thinking stream. Flushes remaining buffer and drains all queued lines.
    pub(crate) fn finalize(&mut self) -> Option<Box<dyn HistoryCell>> {
        // Flush any remaining buffered text.
        if !self.buffer.is_empty() {
            let remaining = std::mem::take(&mut self.buffer);
            let line = Line::from(Span::from(remaining).italic().magenta());
            let now = Instant::now();
            self.queue.push_back(ThinkingQueuedLine {
                line,
                enqueued_at: now,
            });
        }
        let lines: Vec<Line<'static>> = self.queue.drain(..).map(|q| q.line).collect();
        self.emit(lines)
    }

    /// Step animation: commit at most one queued line.
    pub(crate) fn on_commit_tick(&mut self) -> (Option<Box<dyn HistoryCell>>, bool) {
        let lines: Vec<Line<'static>> =
            self.queue.pop_front().map(|q| q.line).into_iter().collect();
        (self.emit(lines), self.queue.is_empty())
    }

    /// Step animation: commit at most `max_lines` queued lines.
    pub(crate) fn on_commit_tick_batch(
        &mut self,
        max_lines: usize,
    ) -> (Option<Box<dyn HistoryCell>>, bool) {
        let end = max_lines.max(1).min(self.queue.len());
        let lines: Vec<Line<'static>> = self.queue.drain(..end).map(|q| q.line).collect();
        (self.emit(lines), self.queue.is_empty())
    }

    /// Returns the current number of queued thinking lines waiting to be displayed.
    pub(crate) fn queued_lines(&self) -> usize {
        self.queue.len()
    }

    /// Returns the age of the oldest queued thinking line.
    pub(crate) fn oldest_queued_age(&self, now: Instant) -> Option<Duration> {
        self.queue
            .front()
            .map(|q| now.saturating_duration_since(q.enqueued_at))
    }

    /// Split the buffer on newlines. Each completed line is styled with italic magenta (no prefix
    /// — indentation is handled by `adaptive_wrap_lines` in `ThinkingStreamCell::display_lines`).
    /// Partial lines (no trailing newline) remain in the buffer.
    fn commit_complete_lines(&mut self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        while let Some(pos) = self.buffer.find('\n') {
            let line_text = self.buffer[..pos].to_string();
            self.buffer = self.buffer[pos + 1..].to_string();
            lines.push(Line::from(Span::from(line_text).italic().magenta()));
        }
        lines
    }

    fn emit(&mut self, lines: Vec<Line<'static>>) -> Option<Box<dyn HistoryCell>> {
        if lines.is_empty() {
            return None;
        }

        let mut out_lines: Vec<Line<'static>> = Vec::new();
        let is_stream_continuation = self.header_emitted;

        if !self.header_emitted {
            out_lines.push(Line::from("Thought:".italic().magenta()));
            self.header_emitted = true;
        }

        out_lines.extend(lines);

        Some(Box::new(history_cell::new_thinking_stream(
            out_lines,
            is_stream_continuation,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_cwd() -> PathBuf {
        // These tests only need a stable absolute cwd; using temp_dir() avoids baking Unix- or
        // Windows-specific root semantics into the fixtures.
        std::env::temp_dir()
    }

    fn lines_to_plain_strings(lines: &[ratatui::text::Line<'_>]) -> Vec<String> {
        lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.clone())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect()
    }

    #[tokio::test]
    async fn controller_loose_vs_tight_with_commit_ticks_matches_full() {
        let mut ctrl = StreamController::new(None, &test_cwd());
        let mut lines = Vec::new();

        // Exact deltas from the session log (section: Loose vs. tight list items)
        let deltas = vec![
            "\n\n",
            "Loose",
            " vs",
            ".",
            " tight",
            " list",
            " items",
            ":\n",
            "1",
            ".",
            " Tight",
            " item",
            "\n",
            "2",
            ".",
            " Another",
            " tight",
            " item",
            "\n\n",
            "1",
            ".",
            " Loose",
            " item",
            " with",
            " its",
            " own",
            " paragraph",
            ".\n\n",
            "  ",
            " This",
            " paragraph",
            " belongs",
            " to",
            " the",
            " same",
            " list",
            " item",
            ".\n\n",
            "2",
            ".",
            " Second",
            " loose",
            " item",
            " with",
            " a",
            " nested",
            " list",
            " after",
            " a",
            " blank",
            " line",
            ".\n\n",
            "  ",
            " -",
            " Nested",
            " bullet",
            " under",
            " a",
            " loose",
            " item",
            "\n",
            "  ",
            " -",
            " Another",
            " nested",
            " bullet",
            "\n\n",
        ];

        // Simulate streaming with a commit tick attempt after each delta.
        for d in deltas.iter() {
            ctrl.push(d);
            while let (Some(cell), idle) = ctrl.on_commit_tick() {
                lines.extend(cell.transcript_lines(u16::MAX));
                if idle {
                    break;
                }
            }
        }
        // Finalize and flush remaining lines now.
        if let Some(cell) = ctrl.finalize() {
            lines.extend(cell.transcript_lines(u16::MAX));
        }

        let streamed: Vec<_> = lines_to_plain_strings(&lines)
            .into_iter()
            // skip • and 2-space indentation
            .map(|s| s.chars().skip(2).collect::<String>())
            .collect();

        // Full render of the same source
        let source: String = deltas.iter().copied().collect();
        let mut rendered: Vec<ratatui::text::Line<'static>> = Vec::new();
        let test_cwd = test_cwd();
        crate::markdown::append_markdown(&source, None, Some(test_cwd.as_path()), &mut rendered);
        let rendered_strs = lines_to_plain_strings(&rendered);

        assert_eq!(streamed, rendered_strs);

        // Also assert exact expected plain strings for clarity.
        let expected = vec![
            "Loose vs. tight list items:".to_string(),
            "".to_string(),
            "1. Tight item".to_string(),
            "2. Another tight item".to_string(),
            "3. Loose item with its own paragraph.".to_string(),
            "".to_string(),
            "   This paragraph belongs to the same list item.".to_string(),
            "4. Second loose item with a nested list after a blank line.".to_string(),
            "    - Nested bullet under a loose item".to_string(),
            "    - Another nested bullet".to_string(),
        ];
        assert_eq!(
            streamed, expected,
            "expected exact rendered lines for loose/tight section"
        );
    }
}
