# .github/codex/home/

Codex home directory configuration for the Codex GitHub Action runtime environment.

## Purpose

Provides the `config.toml` file that configures the Codex CLI when it runs inside the GitHub Action. This sets global defaults such as the model to use.

## Key Files

- **`config.toml`** -- Codex CLI configuration file. Currently sets:
  - `model = "gpt-5.1"` -- The default LLM model used for all Codex Action tasks (triage, review, deduplication, etc.).
  - Includes a comment placeholder for `[mcp_servers]` configuration.

## Plugs Into

- The `openai/codex-action@main` GitHub Action reads this file as the Codex home directory config when executing prompts from `.github/codex/labels/`.
- Changing the `model` value here affects all Codex-powered automation workflows in the repository.

## Imports / Exports

- No code imports. This is a declarative TOML configuration file consumed by the Codex CLI runtime.
