# .codex/skills/babysit-pr/agents/ — Agent Configuration

## What This Folder Does

Contains agent platform configuration files that define how the PR Babysitter skill presents itself when registered with an external agent platform (e.g., OpenAI's agent framework).

## Key Files

| File | Role |
|------|------|
| `openai.yaml` | Defines the agent's display name ("PR Babysitter"), short description, and a `default_prompt` — the initial instruction text sent to the agent when the skill is activated without a custom user prompt. |

## What It Plugs Into

- **Parent skill**: `babysit-pr/SKILL.md` defines the detailed behavior; this YAML defines the interface/presentation layer.
- **Agent platforms**: The YAML is consumed by agent frameworks that need structured metadata about the skill (display name, description, default invocation prompt).

## `openai.yaml` Structure

```yaml
interface:
  display_name: "PR Babysitter"
  short_description: "Watch PR CI, reviews, and merge conflicts"
  default_prompt: "<full default instructions for initial invocation>"
```

The `default_prompt` instructs the agent to:
- Prefer `--watch` mode for live monitoring.
- Fix valid issues and push updates.
- Rerun flaky failures up to 3 times.
- Keep exactly one watcher session active.
- Restart `--watch` after any fix push.
- Continue autonomously until a strict stop condition.

## Relationship to SKILL.md

`SKILL.md` contains the comprehensive behavioral specification (the "how"). `openai.yaml` contains the interface metadata and default invocation prompt (the "what to say when activating"). The YAML's `default_prompt` is a concise summary of SKILL.md's instructions.
