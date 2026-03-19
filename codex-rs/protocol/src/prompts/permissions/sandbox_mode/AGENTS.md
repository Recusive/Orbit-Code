# codex-rs/protocol/src/prompts/permissions/sandbox_mode/

This file applies to `codex-rs/protocol/src/prompts/permissions/sandbox_mode/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-protocol` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-protocol`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Sandbox mode prompt templates -- one per `SandboxMode` variant.

### What this folder does

Contains markdown instructions that describe the active filesystem sandbox mode to the agent. Each file uses a `{network_access}` placeholder that is filled at runtime.

### Key files

- `read_only.md` -- `SandboxMode::ReadOnly`: sandbox permits reading files only; network access status is templated
- `workspace_write.md` -- `SandboxMode::WorkspaceWrite`: sandbox permits reading files and editing within `cwd` and `writable_roots`; editing outside requires approval; network access status is templated
- `danger_full_access.md` -- `SandboxMode::DangerFullAccess`: no filesystem sandboxing; all commands permitted; network access status is templated

### What it plugs into

Selected by `codex-core` based on the session's `SandboxPolicy` configuration. The chosen template is embedded in the system prompt with the `{network_access}` placeholder replaced.

### Exports to

Static markdown content consumed during system prompt assembly.
