# codex-rs/utils/stream-parser/src/

Composable set of incremental streaming text parsers for LLM output processing. Handles stripping citation tags, extracting proposed plan blocks, buffering partial UTF-8, and generic inline hidden tag extraction -- all via the `StreamTextParser` trait.

## Build & Test
```bash
cargo build -p orbit-code-utils-stream-parser
cargo test -p orbit-code-utils-stream-parser
```

## Key Considerations
- No external dependencies (std only). All parsers implement `StreamTextParser` for uniform composition.
- `Utf8StreamParser<P>` wraps any parser to accept raw `&[u8]`, buffering incomplete code points across chunk boundaries.
