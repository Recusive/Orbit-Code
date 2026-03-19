# .codex/skills/babysit-pr/agents/

This file applies to `.codex/skills/babysit-pr/agents/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- These directories define Codex skills and their support files. Keep the runnable instructions, helper scripts, and references aligned so the skill still works end-to-end when invoked.
- Prefer editing the smallest scope that owns the behavior: skill-level docs in the skill root, reusable snippets in `references/`, worker prompts in `agents/`, and executable helpers in `scripts/`.
- Agent prompt files should remain tightly scoped to their delegated task. Do not broaden them unless the surrounding orchestration changes too.

## Validate
- Validate by reading the skill from the top-level `SKILL.md` or directory doc and checking that referenced relative paths still exist.

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

### What This Folder Does

Contains agent platform configuration files that define how the PR Babysitter skill presents itself when registered with an external agent platform (e.g., OpenAI's agent framework).

### Key Files

| File | Role |
|------|------|
| `openai.yaml` | Defines the agent's display name ("PR Babysitter"), short description, and a `default_prompt` — the initial instruction text sent to the agent when the skill is activated without a custom user prompt. |

### What It Plugs Into

- **Parent skill**: `babysit-pr/SKILL.md` defines the detailed behavior; this YAML defines the interface/presentation layer.
- **Agent platforms**: The YAML is consumed by agent frameworks that need structured metadata about the skill (display name, description, default invocation prompt).

### `openai.yaml` Structure

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

### Relationship to SKILL.md

`SKILL.md` contains the comprehensive behavioral specification (the "how"). `openai.yaml` contains the interface metadata and default invocation prompt (the "what to say when activating"). The YAML's `default_prompt` is a concise summary of SKILL.md's instructions.
