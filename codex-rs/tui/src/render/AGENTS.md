# codex-rs/tui/src/render/

This file applies to `codex-rs/tui/src/render/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-tui` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.
- Any user-visible TUI change needs matching snapshot coverage. Mirror behavior in the sibling TUI implementation when the same feature exists there.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-tui`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo insta pending-snapshots -p codex-tui`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Rendering engine for the TUI.

### What this folder does

Provides the core rendering infrastructure: syntax highlighting, line utilities, the `Renderable` trait, and layout helpers (insets, rect extensions). This module is used throughout the TUI wherever content needs to be rendered to the ratatui buffer.

### What it plugs into

- **../chatwidget.rs**, **../bottom_pane/**, **../exec_cell/**, and most other TUI modules import from this module for rendering.
- **../lib.rs**: Re-exports `render_markdown_text()` via the highlight module.
- Uses `syntect` and `two-face` for syntax highlighting with ~250 language grammars and 32 bundled color themes.

### Key files

| File | Role |
|------|------|
| `mod.rs` | Module root; defines `Insets` struct, `RectExt` trait (for applying insets to `Rect`), and declares sub-modules. |
| `highlight.rs` | Syntax highlighting engine wrapping `syntect`. Owns four process-global singletons (`SYNTAX_SET`, `THEME`, `THEME_OVERRIDE`, `ORBIT_HOME`). Provides `highlight_bash_to_lines()`, `highlight_code_to_lines()`, `set_theme_override()`, `set_syntax_theme()`, `current_syntax_theme()`, and `validate_theme_name()`. Inputs exceeding 512 KB or 10,000 lines are rejected to prevent pathological CPU/memory usage. |
| `renderable.rs` | `Renderable` trait -- the core rendering interface requiring `render(area, buf)` and `desired_height(width)`. Also defines `RenderableItem` (owned/borrowed wrapper), `FlexRenderable` (for flexible-height layouts), and helper implementations. |
| `line_utils.rs` | Utility functions for ratatui `Line` manipulation: `line_to_static()` (clone borrowed lines to owned), `push_owned_lines()`, `is_blank_line_spaces_only()`, and `prefix_lines()`. |

### Sub-directories

| Directory | Purpose |
|-----------|---------|
| `snapshots/` | Insta test snapshots for rendering tests (e.g., ANSI color palette). |

### Key design decisions

- The `Renderable` trait is the fundamental rendering contract -- all widgets implement it.
- Syntax highlighting singletons are write-once at startup but the active theme can be swapped at runtime for live preview.
- Large input guardrails (512 KB / 10,000 lines) prevent syntect from degrading TUI performance.
