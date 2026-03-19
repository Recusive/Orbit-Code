# codex-rs/utils/stream-parser/

Crate `codex-utils-stream-parser` -- incremental streaming text parsers for LLM output processing.

## What this folder does

Provides a composable set of streaming parsers that process LLM output text incrementally (chunk by chunk). Handles stripping citation tags, extracting proposed plan blocks, buffering partial UTF-8 code points, and general inline hidden tag extraction. All parsers implement the `StreamTextParser` trait for uniform composition.

## Key types and functions

- `StreamTextParser` trait -- common interface: `push_str(chunk)` and `finish()` returning `StreamTextChunk<T>`
- `StreamTextChunk<T>` -- result type with `visible_text` (safe to render) and `extracted` payloads
- `CitationStreamParser` -- strips `<oai-mem-citation>` tags, extracts citation bodies
- `strip_citations(text)` -- one-shot convenience wrapper
- `ProposedPlanParser` -- strips `<proposed_plan>` blocks, emits `ProposedPlanSegment` variants (Start, Delta, End, Normal)
- `strip_proposed_plan_blocks` / `extract_proposed_plan_text` -- convenience wrappers
- `InlineHiddenTagParser` -- generic parser for arbitrary inline hidden tags defined by `InlineTagSpec`
- `AssistantTextStreamParser` -- composes citation stripping and optional plan parsing in one pass
- `Utf8StreamParser<P>` -- wraps any `StreamTextParser` to accept raw `&[u8]` input, buffering partial UTF-8 code points across chunk boundaries

## Imports from

No external dependencies (std only).

## Exports to

Used by `codex-core` for processing streaming LLM responses before rendering in the TUI.

## Key files

- `Cargo.toml` -- crate metadata (no runtime dependencies)
- `src/lib.rs` -- module declarations and re-exports
- `src/stream_text.rs` -- `StreamTextChunk<T>` and `StreamTextParser` trait
- `src/citation.rs` -- `CitationStreamParser` and `strip_citations`
- `src/proposed_plan.rs` -- `ProposedPlanParser`, `ProposedPlanSegment`, and extraction helpers
- `src/inline_hidden_tag.rs` -- generic `InlineHiddenTagParser` for arbitrary hidden tags
- `src/tagged_line_parser.rs` -- line-oriented tag parser used by `ProposedPlanParser`
- `src/assistant_text.rs` -- `AssistantTextStreamParser` composing citations and plans
- `src/utf8_stream.rs` -- `Utf8StreamParser` with partial code point buffering and error recovery
