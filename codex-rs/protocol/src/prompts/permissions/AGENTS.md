# codex-rs/protocol/src/prompts/permissions/

This file applies to `codex-rs/protocol/src/prompts/permissions/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-protocol` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-protocol`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Permission-related prompt templates for sandbox mode and approval policy instructions.

### What this folder does

Contains markdown templates that describe the active sandbox mode and approval policy to the agent. These are dynamically selected based on session configuration and appended to the base instructions.

### Directory structure

- `approval_policy/` -- instructions for each approval policy variant
- `sandbox_mode/` -- instructions for each filesystem sandbox mode

### What it plugs into

Selected at runtime by `codex-core` based on the session's `AskForApproval` and `SandboxPolicy` configuration. The appropriate markdown files are included in the system prompt to inform the agent about what permissions it has and how to request escalation.

### Exports to

Content is embedded as static strings and consumed during system prompt assembly.
