# codex-rs/tui/frames/

ASCII art animation frame assets for the TUI loading spinners.

## What this folder does

Contains 10 subdirectories, each holding 36 ASCII art text frames (frame_1.txt through frame_36.txt) that are compiled into the `codex-tui` binary at build time via `include_str!()`. These frames drive the animated spinner/loading indicator displayed while the agent is processing.

## What it plugs into

- **src/frames.rs**: Uses the `frames_for!()` macro to embed all frame files at compile time as `[&str; 36]` constant arrays. Each variant (default, codex, openai, blocks, dots, hash, hbars, vbars, shapes, slug) maps to one subdirectory here.
- **src/ascii_animation.rs**: Consumes the frame arrays to drive timed animation rendering in the TUI.

## Subdirectories

| Directory | Description |
|-----------|-------------|
| `default/` | Default animation variant |
| `codex/` | Codex-branded animation |
| `openai/` | OpenAI-branded animation |
| `blocks/` | Block-character animation |
| `dots/` | Dot-based animation |
| `hash/` | Hash-character animation |
| `hbars/` | Horizontal bar animation |
| `vbars/` | Vertical bar animation |
| `shapes/` | Geometric shapes animation |
| `slug/` | Slug-style animation |

## Key details

- Each subdirectory contains exactly 36 frames: `frame_1.txt` through `frame_36.txt`.
- Default frame tick rate is 80ms (`FRAME_TICK_DEFAULT`), yielding a ~2.9 second animation cycle.
- Frames are plain ASCII text, not binary assets.
