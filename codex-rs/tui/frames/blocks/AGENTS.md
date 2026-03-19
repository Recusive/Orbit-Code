# codex-rs/tui/frames/blocks/

This file applies to `codex-rs/tui/frames/blocks/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Treat this directory as golden data or generated/static support material. Keep filenames and relative paths stable unless the owning test, renderer, or generator changes first.
- Prefer updating the producer or the corresponding test scenario before editing files here by hand.
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-tui` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.
- Any user-visible TUI change needs matching snapshot coverage. Mirror behavior in the sibling TUI implementation when the same feature exists there.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-tui`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo insta pending-snapshots -p codex-tui`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Block-character ASCII animation frames.

### What this folder does

Contains 36 ASCII art text frames (`frame_1.txt` through `frame_36.txt`) for the "blocks" animation variant of the TUI loading spinner. Uses block characters for the animation style.

### What it plugs into

- **src/frames.rs**: Embedded at compile time via `include_str!()` into the `FRAMES_BLOCKS` constant array.
- Referenced through `ALL_VARIANTS` in `src/frames.rs` for variant selection.

### Key files

- `frame_1.txt` through `frame_36.txt` -- sequential animation frames, each a multi-line ASCII art string.
