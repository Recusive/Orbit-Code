# Orbit Code — Port & Feature Roadmap

> **Last Updated:** 2026-03-25
> **Purpose:** Track what's needed to fully port Orbit Code to the GUI (Orbit IDE) and ship the CLI as a standalone tool.

---

| Done | Todo | Planned |
|------|------|---------|
| **Claude Model Support** | **Ollama Support** | **Hooks System** |
| Anthropic model metadata pipeline mirroring GPT architecture | Local model provider via Ollama API | Lifecycle hooks (pre-tool, post-tool, session start/end) |
| Claude model IDs resolved, displayed, selectable in TUI | Model discovery for locally-running models | User-configurable hook scripts |
| Provider-specific reasoning labels (Max / Extra High) | No auth — detect running Ollama instance | Engine exists in `hooks/` — needs CLI surface + docs |
| | | |
| **OAuth & Auth for Claude** | **OpenRouter Support** | **Plugin Architecture** |
| Multi-provider auth switching — TUI popup for `/auth` and `/model` | OpenRouter as additional cloud provider | Plugin discovery and loading |
| Inline auth flows: API key, OAuth, device code | API key auth flow | Plugin manifest format |
| OAuth login flow for Anthropic accounts | Model listing from OpenRouter catalog | Skill, command, agent, hook components |
| Account mgmt (login, logout, OAuth code) in app-server v2 | | Plugin settings + per-project config |
| | | |
| **Sub-Agent Model Selection** | **Thinking Content Display** | **Automations** |
| User prompted for model/reasoning before spawning sub-agents | ~~Surface extended thinking in TUI + app-server protocol~~ DONE | Scheduled/recurring agent tasks |
| `request_user_input` ungated — all collaboration modes | ~~Render thinking blocks distinct from response content~~ DONE | Cron-style triggers for automated workflows |
| **Thinking Content Display** | Toggle thinking visibility (expand/collapse) | Remote trigger support for CI/CD |
| Raw thinking tokens stream live in TUI (italic magenta) | | |
| Summary/raw state fully separated | | |
| Replay + finalization handles all edge cases | | |
| | | |
| **TUI Stability** | **Fork Trimming** | **TUI Makeover** |
| Fixed TUI hang and OAuth issues | Remove OpenAI distribution artifacts | Visual refresh of terminal UI |
| Repaired test failures from rename aftermath | Strip OpenAI branding, telemetry, packaging | Improved layout, color system, component hierarchy |
| | Rebrand binary + npm package to Orbit Code | Better tool call visualization |
| | | Status bar redesign |
| | | Responsive layout for narrow terminals |
