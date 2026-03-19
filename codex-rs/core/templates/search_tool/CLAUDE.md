# codex-rs/core/templates/search_tool/

Tool search and tool suggest description templates.

## What this folder does

Provides the description templates for the `tool_search` and `tool_suggest` meta-tools that help the agent discover available tools.

## Key files

| File | Purpose |
|------|---------|
| `tool_description.md` | Description template for the `tool_search` tool -- explains how to search for tools by keyword or description |
| `tool_suggest_description.md` | Description template for the `tool_suggest` tool -- explains how to get tool suggestions based on context |

## Where it plugs into

- Loaded via `include_str!()` in `crate::tools::handlers::tool_search` and `crate::tools::handlers::tool_suggest`
- Defines the tool descriptions shown in the tool registry
