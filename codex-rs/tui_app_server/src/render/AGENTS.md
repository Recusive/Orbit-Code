# codex-rs/tui_app_server/src/render/

This file applies to `codex-rs/tui_app_server/src/render/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-tui-app-server` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.
- Any user-visible TUI change needs matching snapshot coverage. Mirror behavior in the sibling TUI implementation when the same feature exists there.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-tui-app-server`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo insta pending-snapshots -p codex-tui-app-server`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Rendering primitives, syntax highlighting, and layout utilities for the TUI.

### What this folder does

Provides the core rendering infrastructure used throughout the TUI: the `Renderable` trait system for composable widget layout, syntax highlighting via `syntect`, and line manipulation utilities. This module decouples layout and rendering concerns from widget-specific logic.

### What it plugs into

- **All TUI widgets**: Every visual component in the TUI (chat widget, bottom pane, onboarding screens, status cards, etc.) uses the `Renderable` trait and layout utilities from this module.
- **syntect / two_face**: The highlighting engine wraps these libraries for ~250-language syntax highlighting with 32 bundled themes.
- **ratatui**: All rendering targets ratatui `Buffer`, `Rect`, `Line`, and `Span` types.

### Key files

| File | Role |
|------|------|
| `mod.rs` | Module root; declares submodules, defines `Insets` struct for padding/margins, and provides the `RectExt` trait with `inset()` for rect shrinking. |
| `renderable.rs` | `Renderable` trait (render + desired_height + cursor_pos), `RenderableItem` enum (owned/borrowed), `FlexRenderable` for flex-layout children, `ColumnRenderable` for vertical stacking, and helper extensions. |
| `highlight.rs` | Syntax highlighting engine wrapping `syntect` -- process-global singletons for grammar database, active theme, and user preference. Functions: `highlight_code_to_lines()`, `highlight_bash_to_lines()`, `set_theme_override()`, `set_syntax_theme()`, `validate_theme_name()`. Input guardrails reject files over 512 KB or 10,000 lines. |
| `line_utils.rs` | Line manipulation utilities -- `line_to_static()` (clone borrowed lines to owned), `push_owned_lines()`, `is_blank_line_spaces_only()`, `prefix_lines()`, and other helpers. |

### Imports from

- `ratatui` -- `Buffer`, `Rect`, `Line`, `Span`, `Paragraph`, `WidgetRef`.
- `syntect` -- `HighlightLines`, `Theme`, `SyntaxSet`, `Scope`.
- `two_face` -- bundled syntax and theme sets.

### Exports to

- **All crate modules**: `Renderable`, `FlexRenderable`, `RenderableItem`, `ColumnRenderable`, `Insets`, `RectExt`, highlighting functions, line utilities.
- **crate::lib** (public): `render_markdown_text()` re-exported from the library root.
