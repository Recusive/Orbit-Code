# codex-rs/protocol/src/prompts/base_instructions/

This file applies to `codex-rs/protocol/src/prompts/base_instructions/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-protocol` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-protocol`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Base system prompt instructions for the Codex CLI agent.

### What this folder does

Contains the core system prompt that defines the Codex agent's personality, capabilities, behavior guidelines, and formatting rules. This is included in every session regardless of configuration.

### Key files

- `default.md` -- the full base instruction set covering:
  - Agent identity and capabilities (coding agent in Codex CLI)
  - Personality (concise, direct, friendly)
  - AGENTS.md spec (how to respect repo-level instructions)
  - Responsiveness guidelines (preamble messages before tool calls)
  - Planning guidelines (`update_plan` tool usage)
  - Task execution rules (work autonomously, use `apply_patch`, coding best practices)
  - Validation workflow (testing, formatting, iterating)
  - Ambition vs. precision guidelines
  - Progress update cadence
  - Final answer formatting (headers, bullets, monospace, file references, tone)
  - Shell command guidelines (prefer `rg`)

### What it plugs into

Included at compile time into the system prompt assembly logic in `codex-core`. Forms the foundation of every Codex session's developer message.

### Exports to

Content is embedded as a static string and combined with permission/sandbox instructions and user-specific context to form the complete system prompt.
