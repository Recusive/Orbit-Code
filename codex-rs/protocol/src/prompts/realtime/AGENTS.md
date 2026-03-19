# codex-rs/protocol/src/prompts/realtime/

This file applies to `codex-rs/protocol/src/prompts/realtime/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-protocol` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-protocol`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Realtime voice conversation prompt templates.

### What this folder does

Contains markdown instructions that frame the start and end of a realtime (voice) conversation session. These instruct the agent to adapt its behavior for transcript-style input from an intermediary.

### Key files

- `realtime_start.md` -- injected when a realtime conversation begins; instructs the agent that it is operating as a backend executor behind an intermediary, user text is transcript-style (possibly unpunctuated with recognition errors), and responses should be concise and action-oriented
- `realtime_end.md` -- injected when a realtime conversation ends; instructs the agent to resume normal chat behavior with typed text (no more transcript assumptions)

### What it plugs into

Injected into the conversation context by `codex-core` when realtime mode is toggled on/off during a session.

### Exports to

Static markdown content consumed during session message construction.
