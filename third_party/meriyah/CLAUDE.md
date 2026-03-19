# third_party/meriyah/

## Purpose

Contains the license file for the [Meriyah](https://github.com/nicolo-ribaudo/meriyah) JavaScript parser, a third-party dependency used in the Codex project.

## Key Files

| File | Role |
|------|------|
| `LICENSE` | ISC license for Meriyah (Copyright 2019 KFlash and others) |

## What Meriyah Is

Meriyah is a fast, standards-compliant JavaScript/ECMAScript parser. It is used in the Codex execution policy subsystem (`codex-rs/execpolicy/`) for parsing and analyzing JavaScript code to make security decisions about command execution.

## Relationship to Other Directories

- `codex-rs/execpolicy/`: Uses Meriyah's parsing logic (ported/adapted) for JavaScript analysis
- Referenced by the root `NOTICE` file for attribution
