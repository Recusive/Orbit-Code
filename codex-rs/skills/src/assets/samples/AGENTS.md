# codex-rs/skills/src/assets/samples/

This file applies to `codex-rs/skills/src/assets/samples/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Treat this directory as golden data or generated/static support material. Keep filenames and relative paths stable unless the owning test, renderer, or generator changes first.
- Prefer updating the producer or the corresponding test scenario before editing files here by hand.
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-skills` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-skills`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Embedded system skill packages distributed with Codex.

### What this folder does

Contains the built-in skill definitions that ship with every Codex installation. Each subdirectory is a complete skill package with agent configuration, reference documents, scripts, and assets.

### Skill packages

- `openai-docs/` -- OpenAI documentation reference skill with model-specific guides.
- `skill-creator/` -- Meta-skill for creating new Codex skills.
- `skill-installer/` -- Skill for installing skills from GitHub repositories.

### Structure convention

Each skill package follows this layout:
- `SKILL.md` -- skill description and metadata.
- `agents/openai.yaml` -- agent configuration.
- `assets/` -- icons and images.
- `references/` -- reference documentation files.
- `scripts/` -- executable scripts used by the skill.
- `LICENSE.txt` or `license.txt` -- license file.
