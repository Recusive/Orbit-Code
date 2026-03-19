# codex-rs/core/templates/tools/

Tool-specific prompt templates.

## What this folder does

Contains prompt templates for specific tools that require detailed instructions beyond simple descriptions.

## Key files

| File | Purpose |
|------|---------|
| `presentation_artifact.md` | Template for the presentation artifact tool, defining the format and structure for generated presentation artifacts |

## Where it plugs into

- Loaded via `include_str!()` in `crate::tools::handlers::artifacts`
- Provides the detailed instructions for artifact creation tools
