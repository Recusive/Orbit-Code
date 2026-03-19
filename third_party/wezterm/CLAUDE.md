# third_party/wezterm/

## Purpose

Contains the license file for [WezTerm](https://github.com/wez/wezterm), a GPU-accelerated terminal emulator whose code has been adapted for use in the Codex project.

## Key Files

| File | Role |
|------|------|
| `LICENSE` | MIT license for WezTerm (Copyright 2018-Present Wez Furlong) |

## What WezTerm Code Is Used For

Terminal emulation and PTY handling code from WezTerm has been adapted for use in the Codex TUI and exec subsystems. This includes terminal escape sequence processing and pseudo-terminal management.

## Relationship to Other Directories

- `codex-rs/tui/`: The terminal UI uses adapted WezTerm terminal handling code
- `codex-rs/exec/`: The execution subsystem uses PTY handling adapted from WezTerm
- Referenced by the root `NOTICE` file for attribution
