# codex-rs/skills/src/assets/samples/

Embedded system skill packages distributed with Codex.

## What this folder does

Contains the built-in skill definitions that ship with every Codex installation. Each subdirectory is a complete skill package with agent configuration, reference documents, scripts, and assets.

## Skill packages

- `openai-docs/` -- OpenAI documentation reference skill with model-specific guides.
- `skill-creator/` -- Meta-skill for creating new Codex skills.
- `skill-installer/` -- Skill for installing skills from GitHub repositories.

## Structure convention

Each skill package follows this layout:
- `SKILL.md` -- skill description and metadata.
- `agents/openai.yaml` -- agent configuration.
- `assets/` -- icons and images.
- `references/` -- reference documentation files.
- `scripts/` -- executable scripts used by the skill.
- `LICENSE.txt` or `license.txt` -- license file.
