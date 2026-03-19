# codex-rs/protocol/src/prompts/permissions/

Permission-related prompt templates for sandbox mode and approval policy instructions.

## What this folder does

Contains markdown templates that describe the active sandbox mode and approval policy to the agent. These are dynamically selected based on session configuration and appended to the base instructions.

## Directory structure

- `approval_policy/` -- instructions for each approval policy variant
- `sandbox_mode/` -- instructions for each filesystem sandbox mode

## What it plugs into

Selected at runtime by `codex-core` based on the session's `AskForApproval` and `SandboxPolicy` configuration. The appropriate markdown files are included in the system prompt to inform the agent about what permissions it has and how to request escalation.

## Exports to

Content is embedded as static strings and consumed during system prompt assembly.
