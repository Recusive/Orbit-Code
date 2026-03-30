<p align="center">
  <img src="orbit-icon.png" alt="Orbit Code" width="128" />
</p>

<h1 align="center">Orbit Code</h1>

<p align="center">
  <strong>A multi-provider terminal coding agent.</strong><br/>
  Fork of <a href="https://github.com/openai/codex">OpenAI Codex CLI</a> with Anthropic Claude support, multi-provider auth, extended thinking display, and more.
</p>

<p align="center">
  <a href="https://github.com/Recusive/Orbit-Code/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-Apache_2.0-blue" alt="License" /></a>
  <img src="https://img.shields.io/badge/Rust-1.93-DEA584?logo=rust&logoColor=black" alt="Rust" />
  <img src="https://img.shields.io/badge/TypeScript-5.7-3178C6?logo=typescript&logoColor=white" alt="TypeScript" />
  <img src="https://img.shields.io/badge/macOS-Apple%20Silicon-000000?logo=apple&logoColor=white" alt="macOS" />
  <img src="https://img.shields.io/badge/Linux-x86__64%20%7C%20arm64-FCC624?logo=linux&logoColor=black" alt="Linux" />
</p>

---

## Why This Exists

[OpenAI Codex CLI](https://github.com/openai/codex) is a great terminal coding agent — but it only works with OpenAI models. We forked it to build [Orbit](https://orbit.build), an AI-native development environment, and needed the terminal agent to work with **Anthropic Claude** as a first-class provider.

**We've since pivoted away from Orbit and are open-sourcing all of this work.** Everything we built on top of Codex is now free for anyone to use, extend, and contribute to.

---

## What We Added (Features Codex Doesn't Have)

These are the features we built from scratch on top of the upstream Codex codebase:

### Anthropic Claude as a First-Class Provider

Not a wrapper or proxy — a proper provider integration with its own crate (`codex-rs/anthropic/`), model catalog, and API client. Claude models are resolved, displayed, and selectable in the TUI just like OpenAI models.

### Multi-Provider Auth Switching

Switch between OpenAI and Anthropic from inside the TUI. No restart needed.

- `/auth` command opens a provider picker with inline auth flows
- `/model` command shows models from the active provider
- Supports API key entry, OAuth login, and device code flows
- Auth state persisted per-provider — switch back and forth without re-authenticating

### Anthropic Model Metadata Pipeline

A catalog-driven architecture that mirrors what Codex has for GPT models, but for Claude:

- Model IDs resolved and validated against the Anthropic catalog
- Context window sizes, capability flags, and pricing pulled from metadata
- Provider-specific reasoning level labels — **"Max"** for Claude, **"Extra High"** for OpenAI

### Extended Thinking / Reasoning Display

Live streaming of Claude's thinking tokens directly in the TUI:

- Thinking content renders in italic magenta, visually distinct from response text
- Summary vs. raw state fully separated in the protocol
- Replay and finalization handle all edge cases (interrupts, tool calls mid-thought)
- Requests `reasoning.content` from the Responses API so thinking tokens actually reach the TUI

### Sub-Agent Model Selection

When spawning sub-agents, you're prompted to choose the model and reasoning level — rather than inheriting the parent's config blindly.

### Ungated User Input

`request_user_input` works in **all** collaboration modes, not just the default. Sub-agents can ask clarifying questions regardless of the approval pipeline configuration.

### `.orbit` Config Namespace

Replaced `.codex` with `.orbit` for config directories and environment variables (`ORBIT_HOME`). Legacy `.codex` fallback was cleanly removed.

### Upstream Sync to 0.118.0

We merged 292 commits from upstream Codex (0.118.0), resolving 684 conflict files across a 6-phase sync. The approach was add-only — we never removed our fork's logic during the merge.

---

## Everything Else (From Upstream Codex)

All the original Codex features are here too:

- **Rich Terminal UI** — Ratatui-based TUI with syntax highlighting, tool call visualization, and responsive layout
- **Sandboxed Execution** — Platform-specific sandboxes (Seatbelt on macOS, Landlock/seccomp on Linux)
- **MCP Support** — Model Context Protocol server for IDE integrations
- **App Server** — JSON-RPC WebSocket API for programmatic access
- **Session Management** — Persistent sessions with SQLite-backed conversation history
- **Hooks System** — Lifecycle hooks for customizing agent behavior
- **Multi-Agent** — Spawn and manage multiple agent instances
- **Skills Framework** — Extensible skill system for custom agent behaviors
- **Python & TypeScript SDKs** — Programmatic access from your language of choice
- **File Operations** — Read, write, search, and patch files with approval workflows
- **Git Integration** — Built-in git operations and diff handling

---

## Quick Start

```bash
# Clone
git clone https://github.com/Recusive/Orbit-Code.git
cd Orbit-Code

# Install dependencies
cd codex-rs && cargo fetch

# Run from source
just codex

# Run tests
just test

# Format & lint
just fmt
just fix
```

### Prerequisites

- Rust 1.93+ (pinned in `codex-rs/rust-toolchain.toml`)
- Node.js 22+ and pnpm 10+ (for npm wrapper and TypeScript SDK)
- [`just`](https://github.com/casey/just) command runner
- `cargo-nextest` (recommended for faster tests)

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     CONSUMER LAYER                           │
│  tui              Terminal UI (Ratatui)                      │
│  tui_app_server   TUI variant for IDE integration            │
│  cli              Binary entry point                         │
│  app-server       JSON-RPC WebSocket API                     │
│  mcp-server       MCP protocol server                        │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────┐
│                      ENGINE LAYER                            │
│  core              Agent loop, tool execution, config        │
│  anthropic         Anthropic API client (our addition)       │
│  app-server-protocol  JSON-RPC types (v1 + v2)              │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────┐
│                    FOUNDATION LAYER                           │
│  protocol          Op, EventMsg, SandboxPolicy, etc.         │
│  config            TOML config parsing and layer merging     │
│  hooks             Lifecycle hook execution engine            │
│  login             OAuth/auth login flows (multi-provider)   │
│  secrets           Encrypted secrets (keyring backend)        │
│  utils/*           ~20 utility crates                        │
└─────────────────────────────────────────────────────────────┘
```

67+ Rust crates. The workspace is in `codex-rs/`. All `just` commands run from there.

---

## Repository Structure

```
Orbit-Code/
├── codex-rs/              # Primary Rust codebase (67+ crates)
│   ├── anthropic/         # ★ Anthropic Claude API client
│   ├── login/             # ★ Multi-provider auth (OAuth, API key, device code)
│   ├── core/              # Agent engine, tools, config
│   ├── tui/               # Terminal UI (Ratatui)
│   ├── protocol/          # Message types and prompts
│   ├── cli/               # Binary entry point
│   ├── app-server/        # JSON-RPC WebSocket API
│   ├── mcp-server/        # MCP protocol server
│   ├── hooks/             # Lifecycle hook system
│   ├── config/            # TOML config system
│   ├── state/             # SQLite session persistence
│   └── utils/             # ~20 utility crates
├── sdk/
│   ├── python/            # Python SDK
│   └── typescript/        # TypeScript SDK
├── shell-tool-mcp/        # Shell tool MCP server
├── codex-cli/             # npm package wrapper
├── docs/                  # Documentation
└── scripts/               # Build & install scripts
```

Items marked with ★ are our additions to the upstream codebase.

---

## Key Commands

| Command | Description |
|---------|-------------|
| `just codex` | Run Orbit Code from source |
| `just test` | Run all Rust tests (nextest) |
| `just fmt` | Format Rust code |
| `just fix` | Run clippy fixes |
| `just mcp-server-run` | Run the MCP server |
| `just write-config-schema` | Regenerate config schema |
| `just write-app-server-schema` | Regenerate app-server schema |

---

## Relationship to Upstream

This is a fork of [openai/codex](https://github.com/openai/codex). We track upstream and periodically sync (last sync: 0.118.0, 292 commits merged). Our changes are additive — we don't remove upstream functionality.

If you're looking for the OpenAI-only version, use the [upstream repo](https://github.com/openai/codex) directly.

---

## Contributing

We welcome contributions. This is now a community project.

- **Bug fixes and improvements** — PRs welcome
- **New provider integrations** — The multi-provider architecture is designed for this
- **Hooks and plugins** — The hooks system is partially built; help us finish it

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

---

## License

Apache License 2.0. See [LICENSE](LICENSE) for details.

Upstream Codex code is also Apache 2.0 licensed.

---

<p align="center">
  Originally built by <a href="https://orbit.build">Recursive Labs</a> for Orbit.<br/>
  Now open source. Use it, fork it, make it yours.
</p>
