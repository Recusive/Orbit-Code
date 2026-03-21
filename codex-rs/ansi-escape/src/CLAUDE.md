# codex-rs/ansi-escape/src/

Source for the `orbit-code-ansi-escape` crate -- single-file ANSI-to-ratatui conversion.

## Module Layout
- **Single file** (`lib.rs`): `ansi_escape()` for multi-line conversion, `ansi_escape_line()` for single-line conversion, `expand_tabs()` internal helper replacing tabs with 4 spaces
