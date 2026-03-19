# third_party/

This file applies to `third_party/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Read the files listed below before changing behavior in this subtree; keep neighboring docs and call sites consistent with any structural change.

## Validate
- Run the nearest package or crate tests that exercise this subtree.

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### Purpose

Contains license files for vendored third-party code used in the Codex project. Each subdirectory corresponds to an external project whose code has been incorporated or adapted.

### Contents

| Directory | Project | License | Usage |
|-----------|---------|---------|-------|
| `meriyah/` | [Meriyah](https://github.com/nicolo-ribaudo/meriyah) | ISC | JavaScript parser (used in the execpolicy/exec subsystems for analyzing JavaScript code) |
| `wezterm/` | [WezTerm](https://github.com/wez/wezterm) | MIT | Terminal emulator library (code adapted for the TUI terminal handling in `codex-rs/`) |

### Key Files

| File | Role |
|------|------|
| `meriyah/LICENSE` | ISC license for the Meriyah JavaScript parser |
| `wezterm/LICENSE` | MIT license for WezTerm terminal emulator code |

### Relationship to Other Directories

- These licenses correspond to code used in the `codex-rs/` Rust workspace
- The `NOTICE` file at the repo root references these third-party attributions
