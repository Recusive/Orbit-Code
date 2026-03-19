# tools/argument-comment-lint/.cargo/

## Purpose

Cargo configuration for the Dylint argument-comment lint crate.

## Key Files

| File | Role |
|------|------|
| `config.toml` | Sets `dylint-link` as the linker for all targets — required for Dylint lint libraries to load correctly |

## What It Does

The `config.toml` sets `rustflags = ["-C", "linker=dylint-link"]` for all targets (`cfg(all())`). This tells Cargo to use `dylint-link` instead of the default linker when building this crate, which is how Dylint lint libraries get compiled as loadable cdylib plugins.

## Plugs Into

- Parent crate: `tools/argument-comment-lint/` (the Dylint lint library)
- Requires `dylint-link` to be installed (`cargo install dylint-link`)
