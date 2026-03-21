# codex-rs/ansi-escape/

Converts strings containing ANSI escape codes into Ratatui `Text` and `Line` types for TUI rendering. Also handles tab expansion (tabs to 4 spaces).

## Build & Test
```bash
cargo build -p orbit-code-ansi-escape
cargo test -p orbit-code-ansi-escape
```

## Architecture

This is a thin wrapper around the `ansi-to-tui` crate. `ansi_escape()` converts a multi-line ANSI string to `ratatui::text::Text`, and `ansi_escape_line()` does the same for a single line. Tabs are expanded to 4 spaces before conversion to avoid visual artifacts in transcript/gutter views.

## Key Considerations
- `ansi_escape()` panics on parse or UTF-8 errors -- these are treated as programmer errors, not recoverable failures
- `ansi_escape_line()` logs a warning if the input contains multiple lines and returns only the first
- No `[lints]` section in `Cargo.toml` -- workspace lints are not applied to this crate
- This crate has no tests of its own
