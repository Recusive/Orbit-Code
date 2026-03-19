# Stage 1: Rename codex → orbit-code

**Status:** IN PROGRESS

**Full plan:** [01-rename-codex-to-orbit-code-full-plan.md](./01-rename-codex-to-orbit-code-full-plan.md) (also at `docs/orbit/plans/rename-codex-to-orbit-code.md`)

## Summary

Rebrand all internal naming from "codex" to "orbit-code":
- 77 Cargo.toml crate names
- ~800 .rs source files (imports, env vars, strings)
- 76+ BUILD.bazel files
- npm packages → `@orbit.build/orbit-code`
- Python SDK → `orbit-code-app-server-sdk`
- Config paths → `~/.orbit/`, `.orbit/`, `/etc/orbit/`
- Binary → `orbit-code`
- Dual-read shims for backwards compat with `~/.codex/`, `CODEX_API_KEY`, `Codex Auth` keyring

## Preserved Identifiers
- `codex/sandbox-state` MCP wire protocol
- `"codex"` / `"codex-reply"` MCP tool names
- `CODEX_SANDBOX*` env vars and consts
- All `openai.com` provider URLs
- `gpt-*` model name strings
- OpenAI-compatible headers
