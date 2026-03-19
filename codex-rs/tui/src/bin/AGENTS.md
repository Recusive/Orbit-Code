# codex-rs/tui/src/bin/

This file applies to `codex-rs/tui/src/bin/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-tui` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.
- Any user-visible TUI change needs matching snapshot coverage. Mirror behavior in the sibling TUI implementation when the same feature exists there.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-tui`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo insta pending-snapshots -p codex-tui`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Additional binary targets for the `codex-tui` crate.

### What this folder does

Contains auxiliary binary entry points beyond the main `codex-tui` binary (which lives at `../main.rs`). Currently holds a single debugging utility.

### Key files

| File | Role |
|------|------|
| `md-events.rs` | `md-events` binary -- a simple debugging tool that reads Markdown from stdin and prints the `pulldown-cmark` parse events to stdout. Useful for diagnosing markdown rendering issues in the TUI. |

### What it plugs into

- Uses `pulldown-cmark` directly for markdown parsing (same library used by the TUI's markdown renderer).
- Not part of the main application; strictly a developer debugging utility.
