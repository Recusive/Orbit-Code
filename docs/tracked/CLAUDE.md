# Tracked Work

Index of tracked implementation tasks. Update this file whenever items move between stages.

Contains 3 subdirectories (audited, done, todo).

## Done

- `multi-provider-auth-switching-tui-wiring.md` — Phase 1: TUI popup wiring for /auth and /model
- `multi-provider-auth-switching-inline-auth-phase2.md` — Phase 2: Inline auth flows (API key, OAuth, device code)
- `fix-tui-hang-and-oauth.md` — Fix TUI hang and OAuth issues
- `provider-specific-reasoning-level-labels.md` — Provider-specific XHigh labels: "Max" for Claude, "Extra High" for OpenAI
- `ungate-request-user-input.md` — Ungate request_user_input: available in all collaboration modes
- `ask-user-sub-agent-model-selection.md` — Ask user for sub-agent model/reasoning selection before spawning
- `anthropic-model-metadata-pipeline.md` — Proper Anthropic model metadata pipeline mirroring the GPT architecture
- `remove-codex-legacy-go-orbit-only.md` — Remove `.codex` legacy fallback — go `.orbit` only
- `show-thinking-tokens-in-tui.md` — Show thinking tokens live in TUI (italic magenta, plain text) — Claude working, OpenAI blocked by ChatGPT proxy encrypting all reasoning
- `request-reasoning-content-from-api.md` — Request `reasoning.content` from Responses API so thinking tokens reach the TUI

## Todo

- `fork-trimming-remove-openai-distribution.md` — Fork trimming: remove OpenAI distribution
