# codex-rs/tui_app_server/src/bin/

Auxiliary binary targets for the `codex-tui-app-server` crate.

## What this folder does

Contains additional binary entry points declared in `Cargo.toml` alongside the main `codex-tui-app-server` binary. Currently holds a single diagnostic utility.

## What it plugs into

- **Cargo.toml**: Declares `md-events-app-server` as a `[[bin]]` target with path `src/bin/md-events.rs`.

## Key files

| File | Role |
|------|------|
| `md-events.rs` | Diagnostic tool that reads Markdown from stdin, parses it with `pulldown-cmark`, and prints each parser event to stdout. Useful for debugging markdown rendering behavior in the TUI. |

## Imports from

- `pulldown_cmark` (workspace dependency) for markdown parsing.
- Standard library only (`std::io`).

## Exports to

- Standalone binary; not imported by other crates.
