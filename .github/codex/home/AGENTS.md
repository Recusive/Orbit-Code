# .github/codex/home/

This file applies to `.github/codex/home/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Workflow and automation changes should be validated against their callers. Prefer small, explicit changes to job names, permissions, and artifact paths.

## Validate
- No dedicated local build step for this directory; validate by checking the workflows or callers that reference it.

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Codex home directory configuration for the Codex GitHub Action runtime environment.

### Purpose

Provides the `config.toml` file that configures the Codex CLI when it runs inside the GitHub Action. This sets global defaults such as the model to use.

### Key Files

- **`config.toml`** -- Codex CLI configuration file. Currently sets:
  - `model = "gpt-5.1"` -- The default LLM model used for all Codex Action tasks (triage, review, deduplication, etc.).
  - Includes a comment placeholder for `[mcp_servers]` configuration.

### Plugs Into

- The `openai/codex-action@main` GitHub Action reads this file as the Codex home directory config when executing prompts from `.github/codex/labels/`.
- Changing the `model` value here affects all Codex-powered automation workflows in the repository.

### Imports / Exports

- No code imports. This is a declarative TOML configuration file consumed by the Codex CLI runtime.
