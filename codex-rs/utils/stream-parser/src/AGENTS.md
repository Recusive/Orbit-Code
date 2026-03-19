# codex-rs/utils/stream-parser/src/

This file applies to `codex-rs/utils/stream-parser/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-utils-stream-parser` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-utils-stream-parser`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source directory for the `codex-utils-stream-parser` crate.

### Key files

- `lib.rs` -- module declarations and re-exports of all public types
- `stream_text.rs` -- foundational types:
  - `StreamTextChunk<T>` -- holds `visible_text: String` and `extracted: Vec<T>`; has `is_empty()` and `Default`
  - `StreamTextParser` trait -- `type Extracted`; `push_str(&mut self, chunk: &str) -> StreamTextChunk<Self::Extracted>`; `finish(&mut self) -> StreamTextChunk<Self::Extracted>`
- `citation.rs` -- `CitationStreamParser`:
  - Wraps `InlineHiddenTagParser` configured for `<oai-mem-citation>` / `</oai-mem-citation>`
  - Extracts citation bodies as `String`; auto-closes unterminated tags at EOF
  - `strip_citations(text)` -- one-shot convenience function
- `proposed_plan.rs` -- `ProposedPlanParser`:
  - Uses `TaggedLineParser` for `<proposed_plan>` / `</proposed_plan>` (line-oriented matching)
  - Emits `ProposedPlanSegment` variants: `Normal(String)`, `ProposedPlanStart`, `ProposedPlanDelta(String)`, `ProposedPlanEnd`
  - `strip_proposed_plan_blocks` and `extract_proposed_plan_text` convenience functions
- `inline_hidden_tag.rs` -- `InlineHiddenTagParser<T>`:
  - Generic parser supporting multiple tag specs (open/close pairs)
  - Character-by-character state machine that buffers partial tag prefixes
  - Emits `ExtractedInlineTag` with tag identifier and content
- `tagged_line_parser.rs` -- `TaggedLineParser<T>`:
  - Line-oriented parser where tags must appear alone on a line
  - Emits `TaggedLineSegment<T>` variants: `Normal`, `TagStart`, `TagDelta`, `TagEnd`
- `assistant_text.rs` -- `AssistantTextStreamParser`:
  - Composes `CitationStreamParser` then optionally `ProposedPlanParser`
  - Produces `AssistantTextChunk` with `visible_text`, `citations`, and `plan_segments`
- `utf8_stream.rs` -- `Utf8StreamParser<P>`:
  - Wraps any `StreamTextParser` to accept `&[u8]` input
  - Buffers incomplete UTF-8 code points across chunk boundaries
  - Rolls back entire chunks on invalid UTF-8; errors on incomplete code points at EOF
  - `into_inner()` / `into_inner_lossy()` for unwrapping the inner parser
