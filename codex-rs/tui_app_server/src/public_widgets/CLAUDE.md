# codex-rs/tui_app_server/src/public_widgets/

Public reusable UI widgets exported from this crate for use by external crates.

## What this folder does

Provides a public API surface for widgets that external crates can reuse. Currently contains `ComposerInput`, a minimal wrapper around the internal `ChatComposer` that provides a standalone text-input field with submit semantics (Enter to submit, Shift+Enter for newline, paste heuristics, multi-line editing).

## What it plugs into

- **../lib.rs**: Re-exports `ComposerInput` and `ComposerAction` as public API.
- **External crates** (e.g., `codex-cloud-tasks`): Import `ComposerInput` for reusable text input.

## Key files

| File | Role |
|------|------|
| `mod.rs` | Module root; declares the `composer_input` submodule. |
| `composer_input.rs` | `ComposerInput` struct -- public wrapper around `ChatComposer`. `ComposerAction` enum -- submit vs. no-op result from key input. Provides `new()`, `handle_key_event()`, `render()`, and `needs_redraw()`. |

## Imports from

- `crate::bottom_pane::ChatComposer` / `InputResult` -- the internal composer implementation.
- `crate::app_event` / `crate::app_event_sender` -- internal event plumbing (isolated with a dummy channel).
- `crate::render::renderable::Renderable` -- rendering trait.

## Exports to

- **crate::lib** (public): `ComposerInput`, `ComposerAction`.
- **External crates**: Any crate depending on `codex-tui-app-server` can use these widgets.
