# codex-rs/core/src/tools/handlers/

This file applies to `codex-rs/core/src/tools/handlers/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Concrete tool handler implementations for all Codex tools.

### What this folder does

Each handler implements the `ToolHandler` trait for a specific tool, parsing arguments, validating inputs, and executing the tool's logic. Handlers are registered in the `ToolRouter` during session initialization.

### Available tools

| Handler | File | Tool name | Purpose |
|---------|------|-----------|---------|
| `ShellHandler` / `ShellCommandHandler` | `shell.rs` | `shell` | Execute shell commands |
| `UnifiedExecHandler` | `unified_exec.rs` | `exec` | Interactive process execution (open, write_stdin, kill, etc.) |
| `ApplyPatchHandler` | `apply_patch.rs` | `apply_patch` | Apply file patches/diffs |
| `ReadFileHandler` | `read_file.rs` | `read_file` | Read file contents |
| `ListDirHandler` | `list_dir.rs` | `list_dir` | List directory contents |
| `GrepFilesHandler` | `grep_files.rs` | `grep_files` | Search file contents with regex |
| `McpHandler` | `mcp.rs` | `mcp__*` | Execute MCP server tool calls |
| `McpResourceHandler` | `mcp_resource.rs` | `list_mcp_resources` etc. | MCP resource operations |
| `JsReplHandler` / `JsReplResetHandler` | `js_repl.rs` | `js_repl` | JavaScript REPL execution |
| `ToolSearchHandler` | `tool_search.rs` | `tool_search` | Search available tools by keyword |
| `ToolSuggestHandler` | `tool_suggest.rs` | `tool_suggest` | Suggest relevant tools |
| `ViewImageHandler` | `view_image.rs` | `view_image` | View image files |
| `PlanHandler` | `plan.rs` | `plan` | Planning tool for task decomposition |
| `ArtifactsHandler` | `artifacts.rs` | `create_artifact` | Create presentation artifacts |
| `RequestPermissionsHandler` | `request_permissions.rs` | `request_permissions` | Request additional sandbox permissions |
| `RequestUserInputHandler` | `request_user_input.rs` | `request_user_input` | Request input from the user |
| `DynamicToolHandler` | `dynamic.rs` | (dynamic) | Handles dynamically registered tools |
| `TestSyncHandler` | `test_sync.rs` | `test_sync` | Test synchronization (testing only) |
| Multi-agent handlers | `multi_agents/` | `spawn_agent`, `send_input`, etc. | Multi-agent orchestration |
| Agent job handlers | `agent_jobs.rs` | `spawn_agent_job`, etc. | Background agent jobs |

### Key files

| File | Purpose |
|------|---------|
| `mod.rs` | Module declarations, re-exports, `parse_arguments()` helper, additional permissions validation |
| `shell.rs` | Shell command execution with sandbox support |
| `unified_exec.rs` | Interactive process management |
| `apply_patch.rs` | File patch application |
| `mcp.rs` | MCP tool call forwarding |
| `tool_search.rs` | Tool search by keyword/description |
| `multi_agents/` | Sub-agent spawn/resume/close/send/wait handlers |

### Imports from

- `crate::codex` -- `Session`, `TurnContext`
- `crate::tools::sandboxing` -- `ToolCtx`, `ApprovalCtx`, `ToolRuntime`
- `crate::sandboxing` -- `SandboxPermissions`, `PermissionProfile`
- `crate::function_tool` -- `FunctionCallError`

### Exports to

- `crate::tools::registry` -- Handlers registered in `ToolRouter`
- `crate::tools::router` -- Handlers dispatched by `ToolRouter`
