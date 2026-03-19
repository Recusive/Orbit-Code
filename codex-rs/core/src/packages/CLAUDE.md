# codex-rs/core/src/packages/

Pinned package versions for runtime dependencies.

## What this folder does

Provides version constants for packages that are installed via package managers at runtime. This ensures consistent, reproducible installations across different environments.

## Key files

| File | Purpose |
|------|---------|
| `mod.rs` | Module declaration for `versions` |
| `versions.rs` | `ARTIFACT_RUNTIME` constant (currently `"2.5.6"`) -- pinned version for the artifact runtime package |

## Imports from

None (leaf module).

## Exports to

- `crate::mcp::skill_dependencies` -- uses version constants when installing MCP server dependencies
- `crate::tools::js_repl` -- may reference runtime versions for REPL setup
