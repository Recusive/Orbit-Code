# codex-rs/skills/src/assets/samples/openai-docs/

This file applies to `codex-rs/skills/src/assets/samples/openai-docs/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

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

OpenAI documentation reference skill.

### What this folder does

Provides reference documentation about OpenAI models and APIs as a built-in Codex skill. Includes guides for prompting and upgrading to newer models.

### Key files

- `SKILL.md` -- skill metadata and description.
- `LICENSE.txt` -- license file.
- `agents/openai.yaml` -- agent configuration for this skill.
- `assets/` -- skill icons (openai.png, openai-small.svg).
- `references/` -- reference markdown documents (model guides, prompting guides).
