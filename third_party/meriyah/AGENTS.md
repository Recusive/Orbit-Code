# third_party/meriyah/

This file applies to `third_party/meriyah/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Read the files listed below before changing behavior in this subtree; keep neighboring docs and call sites consistent with any structural change.

## Validate
- Run the nearest package or crate tests that exercise this subtree.

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### Purpose

Contains the license file for the [Meriyah](https://github.com/nicolo-ribaudo/meriyah) JavaScript parser, a third-party dependency used in the Codex project.

### Key Files

| File | Role |
|------|------|
| `LICENSE` | ISC license for Meriyah (Copyright 2019 KFlash and others) |

### What Meriyah Is

Meriyah is a fast, standards-compliant JavaScript/ECMAScript parser. It is used in the Codex execution policy subsystem (`codex-rs/execpolicy/`) for parsing and analyzing JavaScript code to make security decisions about command execution.

### Relationship to Other Directories

- `codex-rs/execpolicy/`: Uses Meriyah's parsing logic (ported/adapted) for JavaScript analysis
- Referenced by the root `NOTICE` file for attribution
