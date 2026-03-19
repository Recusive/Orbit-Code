# codex-rs/tui/frames/default/

Default ASCII animation frames.

## What this folder does

Contains 36 ASCII art text frames (`frame_1.txt` through `frame_36.txt`) for the "default" animation variant of the TUI loading spinner. This is the standard animation used when no specific variant is selected.

## What it plugs into

- **src/frames.rs**: Embedded at compile time via `include_str!()` into the `FRAMES_DEFAULT` constant array.
- First entry in `ALL_VARIANTS` in `src/frames.rs`, making it the default animation.

## Key files

- `frame_1.txt` through `frame_36.txt` -- sequential animation frames, each a multi-line ASCII art string.
