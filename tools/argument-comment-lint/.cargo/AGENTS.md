# tools/argument-comment-lint/.cargo/

This file applies to `tools/argument-comment-lint/.cargo/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Read the files listed below before changing behavior in this subtree; keep neighboring docs and call sites consistent with any structural change.

## Validate
- Run the nearest package or crate tests that exercise this subtree.

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### Purpose

Cargo configuration for the Dylint argument-comment lint crate.

### Key Files

| File | Role |
|------|------|
| `config.toml` | Sets `dylint-link` as the linker for all targets — required for Dylint lint libraries to load correctly |

### What It Does

The `config.toml` sets `rustflags = ["-C", "linker=dylint-link"]` for all targets (`cfg(all())`). This tells Cargo to use `dylint-link` instead of the default linker when building this crate, which is how Dylint lint libraries get compiled as loadable cdylib plugins.

### Plugs Into

- Parent crate: `tools/argument-comment-lint/` (the Dylint lint library)
- Requires `dylint-link` to be installed (`cargo install dylint-link`)
