# codex-rs/ansi-escape/

This file applies to `codex-rs/ansi-escape/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-ansi-escape` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-ansi-escape`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Crate: `codex-ansi-escape` -- ANSI escape sequence parser for TUI rendering.

### What this crate does

Converts strings containing ANSI escape codes (colors, bold, underline, etc.) into Ratatui `Text` and `Line` types suitable for rendering in the terminal UI. Also handles tab expansion (replacing tabs with 4 spaces) to avoid visual artifacts in transcript views.

### Main functions

- `ansi_escape(s: &str) -> Text<'static>` -- Parses a string with ANSI escapes into a Ratatui `Text` (multi-line)
- `ansi_escape_line(s: &str) -> Line<'static>` -- Parses a single-line string with ANSI escapes; warns if multiple lines are found

### What it plugs into

- Used by `codex-tui` for rendering command output and transcript content that may contain ANSI color codes

### Imports from / exports to

**Dependencies:**
- `ansi-to-tui` -- Third-party crate that does the actual ANSI-to-Ratatui conversion
- `ratatui` -- TUI framework; this crate produces Ratatui `Text` and `Line` types
- `tracing` -- For logging warnings/errors during parsing

**Exported to:**
- Consumed as `codex-ansi-escape` by other workspace crates (primarily `codex-tui`)

### Key files

- `Cargo.toml` -- Crate manifest
- `src/lib.rs` -- Single-file implementation with `ansi_escape` and `ansi_escape_line` functions
