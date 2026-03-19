# codex-rs/core/src/apps/

This file applies to `codex-rs/core/src/apps/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Renders instruction text for the Apps (Connectors) feature that integrates external services via MCP.

### What this folder does

Provides a single function `render_apps_section()` that generates the system prompt section describing how the AI agent should interact with installed apps/connectors. Apps are backed by MCP tool servers and can be triggered explicitly via `[$app-name](app://connector_id)` syntax or implicitly through context.

### Key files

| File | Purpose |
|------|---------|
| `mod.rs` | Module declaration, re-exports `render_apps_section` |
| `render.rs` | `render_apps_section()` -- generates the XML-tagged apps instruction block |

### Imports from

- `crate::mcp::CODEX_APPS_MCP_SERVER_NAME` -- the MCP server name for apps
- `codex_protocol::protocol` -- XML tag constants for apps instructions

### Exports to

- `crate::codex` -- called during system prompt construction to include apps documentation
