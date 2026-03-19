# Orbit Code — Master Migration Plan

> From OpenAI Codex fork to fully independent, multi-provider Orbit Code CLI.

---

## Overview

This repo is a fork of [OpenAI Codex CLI](https://github.com/openai/codex). The goal is to transform it into **Orbit Code** — a multi-provider terminal coding agent that works with Claude, OpenAI, Gemini, OpenRouter, Groq, Mistral, DeepSeek, Ollama, LM Studio, and any OpenAI-compatible endpoint.

The migration is split into 8 stages. Each stage produces a working, testable codebase.

---

## Stages

| Stage | Name | Status | Description |
|-------|------|--------|-------------|
| **1** | [Rename](./01-rename-codex-to-orbit-code.md) | **IN PROGRESS** | Rebrand codex → orbit-code (crate names, imports, binary, config paths, npm/Python packages) |
| **2** | [Remove Dead Crates](./02-remove-dead-crates.md) | PLANNED | Strip OpenAI-only crates (chatgpt, backend-client, cloud-tasks, connectors, responses-api-proxy) |
| **3** | [Multi-Provider Client Layer](./03-multi-provider-clients.md) | PLANNED | Build 3 new Rust HTTP clients: Chat Completions, Anthropic Messages, Google GenerateContent |
| **4** | [Provider Registry](./04-provider-registry.md) | PLANNED | Port provider configs, model definitions, and capabilities from Agent-backend TypeScript to Rust |
| **5** | [Auth System](./05-auth-system.md) | PLANNED | Multi-provider API key storage + OAuth flows for Claude, GitHub Copilot, GitLab |
| **6** | [Message Normalization](./06-message-normalization.md) | PLANNED | Per-provider quirks: Anthropic tool IDs, Mistral padding, Google content format, cache hints |
| **7** | [Core Integration](./07-core-integration.md) | PLANNED | Wire providers into core engine: WireProtocol enum, provider routing, model selection |
| **8** | [TUI & Config Updates](./08-tui-config-updates.md) | PLANNED | Provider/model picker in TUI, auth flow UI, config schema updates |

---

## Architecture: Before & After

### Before (Current — OpenAI Only)

```
User → TUI → core → codex-api (Responses API) → api.openai.com
                         ↓
                   codex-client (HTTP transport)
```

### After (Target — Multi-Provider)

```
User → TUI → core → provider router
                         ├→ orbit-chat-completions → OpenRouter, Groq, Mistral, DeepSeek, ...
                         ├→ orbit-anthropic        → Claude (Haiku, Sonnet, Opus)
                         ├→ orbit-gemini           → Gemini, Vertex AI
                         └→ orbit-responses        → OpenAI Responses API (kept for compat)
                              ↓
                        orbit-code-client (shared HTTP transport, already exists)
```

---

## What Stays vs What Goes vs What's New

### KEEP (rename only)
- `tui/` — Terminal UI (Ratatui) — the crown jewel
- `tui_app_server/` — TUI with app-server backend
- `core/` — Agent engine (refactor the backend seam)
- `protocol/` — Message types
- `app-server/` + `app-server-protocol/` — JSON-RPC WebSocket API
- `exec/` + `exec-server/` — Headless execution
- `cli/` — Binary entry point
- `hooks/` — Lifecycle hooks
- `skills/` — Skills framework
- `state/` — SQLite persistence
- `config/` — TOML config system
- `login/` — OAuth machinery (swap endpoints)
- All sandbox crates (linux-sandbox, windows-sandbox-rs, execpolicy)
- All utils/ crates (19 utility crates)
- `mcp-server/` + `rmcp-client/` — MCP integration
- `file-search/`, `apply-patch/`, `shell-command/`, `shell-escalation/`
- `otel/`, `feedback/`, `ansi-escape/`, `environment/`
- `codex-api/` — KEEP and rename to `orbit-code-api/` (Responses API client, still needed for OpenAI provider)
- `codex-client/` — KEEP (generic HTTP transport)
- `lmstudio/`, `ollama/` — KEEP (local model providers)

### REMOVE (Stage 2)
- `chatgpt/` — ChatGPT backend task management
- `backend-client/` — OpenAI backend API client
- `codex-backend-openapi-models/` — OpenAI API types
- `connectors/` — ChatGPT app directory
- `responses-api-proxy/` — API key injection proxy (Rust binary handles secrets directly)
- `cloud-requirements/` — OpenAI enterprise config enforcement
- `cloud-tasks/` — Cloud task browser TUI
- `cloud-tasks-client/` — Cloud task API client

### NEW (Stages 3-8)
- `orbit-chat-completions/` — /v1/chat/completions client (~600 lines)
- `orbit-anthropic/` — /v1/messages client (~500 lines)
- `orbit-gemini/` — /generateContent client (~500 lines)
- `orbit-providers/` — Provider trait, registry, model definitions (~1500 lines)
- Updates to `core/`, `login/`, `tui/`, `config/`

---

## Provider Coverage (Target)

| Provider | Wire Protocol | Auth | Status |
|----------|--------------|------|--------|
| **OpenAI** | Responses API (existing) | API key, OAuth | Keep existing |
| **Anthropic (Claude)** | /v1/messages | API key, OAuth | Stage 3 |
| **Google Gemini** | /generateContent | API key | Stage 3 |
| **OpenRouter** | /v1/chat/completions | API key | Stage 3 (via Chat Completions) |
| **Groq** | /v1/chat/completions | API key | Stage 3 (via Chat Completions) |
| **Mistral** | /v1/chat/completions | API key | Stage 3 (via Chat Completions) |
| **DeepSeek** | /v1/chat/completions | API key | Stage 3 (via Chat Completions) |
| **xAI (Grok)** | /v1/chat/completions | API key | Stage 3 (via Chat Completions) |
| **Together AI** | /v1/chat/completions | API key | Stage 3 (via Chat Completions) |
| **Perplexity** | /v1/chat/completions | API key | Stage 3 (via Chat Completions) |
| **Cerebras** | /v1/chat/completions | API key | Stage 3 (via Chat Completions) |
| **DeepInfra** | /v1/chat/completions | API key | Stage 3 (via Chat Completions) |
| **Azure OpenAI** | /v1/chat/completions | API key | Stage 3 (via Chat Completions) |
| **Alibaba/Qwen** | /v1/chat/completions | API key | Stage 3 (via Chat Completions) |
| **GLM/Zhipu** | /v1/chat/completions | API key | Stage 3 (via Chat Completions) |
| **Ollama** | /v1/chat/completions | None (local) | Keep existing + Chat Completions |
| **LM Studio** | /v1/chat/completions | None (local) | Keep existing + Chat Completions |
| **GitHub Copilot** | /v1/chat/completions | OAuth (device flow) | Stage 5 |
| **GitLab** | /v1/chat/completions | OAuth | Stage 5 |
| **Google Vertex** | /generateContent | Service account / OAuth | Stage 5 |
| **Amazon Bedrock** | Custom | IAM credentials | Future |

---

## Reference: Agent-backend (TypeScript)

The existing TypeScript agent-backend at `/Users/no9labs/Developer/Recursive/Snowflake-v0/Agent-backend` serves as the specification for:
- Provider configs and model definitions (`provider/provider.ts`, `provider/models.ts`)
- Per-provider message normalization quirks (`provider/transform.ts`)
- Auth flows (OAuth, API keys, well-known) (`auth/service.ts`)
- Tool system (`tool/registry.ts`)
- Streaming pipeline (`session/llm.ts`, `session/processor.ts`)

We port the **knowledge** (which HTTP calls, what quirks, what auth flows) into Rust, not the code.

---

## Timeline Estimate

| Stage | Effort | Depends On |
|-------|--------|------------|
| 1. Rename | 1 day | — |
| 2. Remove dead crates | 0.5 day | Stage 1 |
| 3. Multi-provider clients | 3-4 days | Stage 2 |
| 4. Provider registry | 1-2 days | Stage 3 |
| 5. Auth system | 2 days | Stage 4 |
| 6. Message normalization | 1-2 days | Stage 3 |
| 7. Core integration | 2-3 days | Stages 3-6 |
| 8. TUI & config | 2-3 days | Stage 7 |

**Total: ~2-3 weeks** for a working multi-provider Orbit Code CLI.
