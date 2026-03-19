# .codex/skills/ — Skill Registry

## What This Folder Does

Contains all registered Codex skills. Each subdirectory is a standalone skill that the Codex agent can activate in response to user requests. This directory acts as the skill registry — the agent scans it to discover available skills.

## Current Skills

| Skill | Purpose |
|-------|---------|
| `babysit-pr/` | Continuously monitors a GitHub PR: polls CI status, triages failures (branch-related vs. flaky), processes review comments, manages retry budgets, and pushes fixes autonomously until the PR is ready to merge or requires human intervention. |
| `test-tui/` | Provides instructions for interactively testing the Codex TUI (terminal UI), including how to launch it, set logging, and send test messages. |

## What It Plugs Into

- **Parent**: `.codex/` — the Codex configuration root.
- **Consumer**: `codex-rs/skills/` — the Rust skill loader crate that discovers and reads skill definitions from this directory.
- **Runtime**: The Codex CLI agent matches user intent against each skill's `SKILL.md` front matter (`name` and `description` fields) to decide which skill to activate.

## Skill Directory Convention

Each skill subdirectory must contain a `SKILL.md` file with YAML front matter:

```yaml
---
name: <skill-name>
description: <when to activate this skill>
---
```

The Markdown body of `SKILL.md` contains the full instructions the agent follows. Optional subdirectories (`agents/`, `references/`, `scripts/`) provide supporting resources.

## Adding a New Skill

1. Create a new directory under `.codex/skills/<skill-name>/`.
2. Add a `SKILL.md` with front matter and instructions.
3. Optionally add `agents/`, `references/`, and/or `scripts/` subdirectories.
4. The skill will be automatically discovered on the next agent invocation.
