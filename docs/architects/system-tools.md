# System & Tools Parity Analysis: Orbit Code vs Claude Code

> **Date:** March 20, 2026
> **Source:** Piebald AI's [claude-code-system-prompts](https://github.com/Piebald-AI/claude-code-system-prompts) repo (v2.1.80) + Orbit Code codebase analysis
> **Purpose:** Map every tool and system prompt section between Claude Code and Orbit Code to guide the migration toward Claude Code-level tool use quality
> **Resume session:** `claude --resume 61eb1649-ce74-4b85-a971-3bdd6e23adf2`

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Architecture Comparison](#architecture-comparison)
3. [Tool Inventory: Orbit Code](#tool-inventory-orbit-code)
4. [Tool Inventory: Claude Code](#tool-inventory-claude-code)
5. [Parity Matrix](#parity-matrix)
6. [Experimental Tools Detail](#experimental-tools-detail)
7. [System Prompt Comparison](#system-prompt-comparison)
8. [Tool Description Patterns](#tool-description-patterns)
9. [Key Prompt Engineering Patterns Claude Code Uses](#key-prompt-engineering-patterns-claude-code-uses)
10. [Migration Recommendations](#migration-recommendations)

---

## Executive Summary

Orbit Code (forked from OpenAI Codex CLI) has **2 active tools** exposed to the model (`shell` + `apply_patch`) while Claude Code exposes **20+ dedicated tools**. However, Orbit Code already has **3 production-ready tool handlers** (`read_file`, `grep_files`, `list_dir`) hidden behind an experimental flag that is **never enabled** for any model in `models.json`.

The performance gap between the two systems is primarily driven by:
1. **Tool specialization** — Claude Code gives the model narrow, purpose-built tools instead of one general shell
2. **Prompt engineering** — Claude Code's tool descriptions contain behavioral instructions, not just API docs
3. **Negative routing** — Claude Code explicitly tells the model what NOT to use each tool for
4. **Redundancy** — Critical routing rules appear 3+ times across different prompt sections

---

## Architecture Comparison

| Aspect | Orbit Code | Claude Code |
|--------|-----------|-------------|
| **Product** | Fork of OpenAI Codex CLI | Anthropic's CLI for Claude |
| **Language** | Rust (67+ crates) | TypeScript (compiled, minified) |
| **Model target** | GPT-5.x (migrating to multi-provider) | Claude (Sonnet/Opus/Haiku) |
| **Tool count (active)** | 2 core + ~15 supporting | ~8 loaded + ~15 deferred |
| **Tool system** | `ToolHandler` trait, `ToolRegistry`, `ToolRouter` | Modular JS, conditional assembly |
| **File editing** | `apply_patch` (diff hunks) | `Edit` (exact string replace) + `Write` (new files) |
| **Search** | `shell("rg ...")` | Dedicated `Grep` + `Glob` tools |
| **Planning** | `update_plan` tool | `EnterPlanMode`/`ExitPlanMode` + plan files |
| **Sub-agents** | `spawn_agent`/`wait_agent`/`send_input` | `Agent` tool with typed subagents |
| **Approval model** | Guardian AI risk assessment + sandbox modes | Permission modes + hooks + sandbox |
| **Prompt structure** | Monolithic `.md` files per model | 247 modular pieces, conditionally assembled |
| **Tool descriptions** | Minimal (API reference style) | Extensive (behavioral instructions) |

---

## Tool Inventory: Orbit Code

### Active Tools (always available)

| Tool | Handler File | Lines | Description |
|------|-------------|------:|-------------|
| `shell` | `handlers/shell.rs` | 499 | Execute shell commands (legacy aliases: `container.exec`, `local_shell`) |
| `exec_command` | `handlers/unified_exec.rs` | ~500 | Interactive PTY execution with stdin support |
| `apply_patch` | `handlers/apply_patch.rs` | 466 | File creation/deletion/patching via diff-like format |
| `update_plan` | `handlers/plan.rs` | 153 | Step-by-step plan tracking (pending/in_progress/completed) |
| `request_user_input` | `handlers/request_user_input.rs` | 125 | Ask user multiple-choice questions |
| `request_permissions` | `handlers/request_permissions.rs` | 74 | Request sandbox permission changes |
| `tool_search` | `handlers/tool_search.rs` | 194 | Search MCP/connector tools by keyword |
| `tool_suggest` | `handlers/tool_suggest.rs` | 320 | Get tool suggestions based on context |
| `view_image` | `handlers/view_image.rs` | 230 | Display images to the user |
| `mcp` | `handlers/mcp.rs` | 58 | Execute MCP server tool calls |
| `mcp_resource` | `handlers/mcp_resource.rs` | 667 | List/read MCP resources |
| `dynamic` | `handlers/dynamic.rs` | 134 | Dynamic tool execution |

### Feature-Flagged Tools (enabled by config/model)

| Tool | Handler | Condition | Description |
|------|---------|-----------|-------------|
| `js_repl` / `js_repl_reset` | `handlers/js_repl.rs` | `js_repl_enabled` | JavaScript REPL with persistent state |
| `code` / `code.wait` | `code_mode/` | `code_mode_enabled` | Persistent code execution (Node.js runtime) |
| `artifacts` | `handlers/artifacts.rs` | `Feature::Artifact` | Build presentation artifacts |
| `spawn_agent` / `wait_agent` / `send_input` / `close_agent` / `resume_agent` | `handlers/multi_agents.rs` | `collab_tools` | Multi-agent orchestration |
| `spawn_agents_on_csv` / `report_agent_job_result` | `handlers/agent_jobs.rs` | `agent_jobs_tools` | Batch agent job execution |

### Experimental Tools (built but NEVER enabled)

These handlers are fully implemented with tests but gated behind `experimental_supported_tools` in `models.json`, which is `[]` for every model.

| Tool | Handler File | Lines | Test File | Test Lines | Description |
|------|-------------|------:|-----------|----------:|-------------|
| `read_file` | `handlers/read_file.rs` | 489 | `read_file_tests.rs` | ~400 | Read files with line numbers, offset/limit, indentation-aware block mode |
| `grep_files` | `handlers/grep_files.rs` | 176 | `grep_files_tests.rs` | ~100 | Regex file search via `rg`, sorted by modification time |
| `list_dir` | `handlers/list_dir.rs` | 271 | `list_dir_tests.rs` | ~200 | Directory listing with depth, offset/limit, type labels |
| `test_sync_tool` | `handlers/test_sync.rs` | 154 | — | — | Test-only synchronous handler |

### How experimental tools are gated

In `models.json`, every model has:
```json
"experimental_supported_tools": []
```

In `spec.rs`, registration is conditional:
```rust
if config.experimental_supported_tools.contains(&"grep_files".to_string()) {
    let grep_files_handler = Arc::new(GrepFilesHandler);
    push_tool_spec(&mut builder, create_grep_files_tool(), true, config.code_mode_enabled);
    builder.register_handler("grep_files", grep_files_handler);
}
```

To enable, either:
1. Add tool names to `experimental_supported_tools` array in `models.json` for specific models
2. Remove the conditional gate in `spec.rs` to always register them

---

## Tool Inventory: Claude Code

### Always-Loaded Tools (~8)

| Tool | Description | Orbit Code Equivalent |
|------|-------------|----------------------|
| `Bash` | Execute shell commands with extensive behavioral rules | `shell` / `exec_command` |
| `Read` | Read files with line numbers, offset/limit, PDF, images, notebooks | `read_file` (experimental) |
| `Write` | Create new files or complete rewrites | `apply_patch` (Add File) |
| `Edit` | Exact string find-and-replace in files | `apply_patch` (Update File) |
| `Glob` | Fast file pattern matching (e.g. `**/*.rs`) | **None** (uses `shell("rg --files")`) |
| `Grep` | Regex content search built on ripgrep | `grep_files` (experimental) |
| `Agent` | Launch typed subagents (Explore, Plan, general-purpose, etc.) | `spawn_agent` + `wait_agent` |
| `Skill` | Execute user-defined skills/slash commands | **None** |
| `ToolSearch` | Fetch deferred tool schemas on demand | `tool_search` (different purpose) |

### Deferred Tools (loaded on demand via ToolSearch)

| Tool | Description | Orbit Code Equivalent |
|------|-------------|----------------------|
| `TodoWrite` | Structured task list for tracking work | `update_plan` (similar) |
| `AskUserQuestion` | Ask user questions with options | `request_user_input` |
| `EnterPlanMode` | Transition to planning mode | Collaboration mode: `plan.md` |
| `ExitPlanMode` | Submit plan for user approval | `<proposed_plan>` block |
| `WebSearch` | Search the web for current information | **None** |
| `WebFetch` | Fetch and analyze web page content | **None** |
| `SendMessage` | Send messages in agent teams | `send_input` |
| `TeamCreate` | Create agent teams | **None** (simpler agent model) |
| `TeamDelete` | Delete agent teams | **None** |
| `TaskCreate` | Create tasks in team task lists | **None** |
| `TaskUpdate` | Update task status/ownership | **None** |
| `TaskList` | List tasks and status | **None** |
| `CronCreate` | Schedule recurring prompts | **None** |
| `CronDelete` | Cancel scheduled prompts | **None** |
| `LSP` | Language Server Protocol operations | **None** |
| `NotebookEdit` | Edit Jupyter notebook cells | **None** |
| `EnterWorktree` | Create isolated git worktree | **None** |
| `ExitWorktree` | Exit and optionally clean up worktree | **None** |
| `Computer` | Browser automation (mouse, keyboard, screenshots) | **None** (has MCP browser tools) |

---

## Parity Matrix

### File Operations

| Capability | Claude Code | Orbit Code | Gap |
|-----------|------------|-----------|-----|
| Read file with line numbers | `Read` (always loaded) | `read_file` (experimental, never enabled) | **Enable experimental** |
| Read images | `Read` (multimodal) | `view_image` (active) | Parity |
| Read PDFs | `Read` (with pages param) | None | **Build or skip** |
| Read Jupyter notebooks | `Read` | None | **Build or skip** |
| Write new file | `Write` tool | `apply_patch` (Add File) | Functional parity |
| Edit file (string replace) | `Edit` tool | `apply_patch` (Update File) | Different approach, both work |
| Edit notebook cell | `NotebookEdit` | None | Gap |

### Search Operations

| Capability | Claude Code | Orbit Code | Gap |
|-----------|------------|-----------|-----|
| Find files by glob pattern | `Glob` tool | `shell("rg --files --glob ...")` | **Build new handler** |
| Search file contents (regex) | `Grep` tool | `grep_files` (experimental, never enabled) | **Enable experimental** |
| List directory contents | `Bash("ls")` | `list_dir` (experimental, never enabled) | **Enable experimental** |

### Execution

| Capability | Claude Code | Orbit Code | Gap |
|-----------|------------|-----------|-----|
| Run shell commands | `Bash` | `shell` / `exec_command` | Parity (different descriptions) |
| Run JavaScript | None (separate from main tools) | `js_repl` / `code_mode` | Orbit has MORE |
| Sandbox enforcement | Bash sandbox rules (17 sub-sections) | Guardian AI + seatbelt/landlock | Different approach |

### Planning & Tasks

| Capability | Claude Code | Orbit Code | Gap |
|-----------|------------|-----------|-----|
| Create plan | `EnterPlanMode` → plan file → `ExitPlanMode` | `update_plan` (inline steps) | Different design |
| Track task progress | `TodoWrite` (detailed with examples) | `update_plan` (step statuses) | Functional parity |
| Team task management | `TaskCreate`/`TaskUpdate`/`TaskList` | None (simpler agent model) | Gap (may not need) |

### Agent/Sub-agent

| Capability | Claude Code | Orbit Code | Gap |
|-----------|------------|-----------|-----|
| Spawn sub-agent | `Agent` tool | `spawn_agent` | Parity |
| Typed sub-agents | 10+ types (Explore, Plan, code-reviewer...) | Untyped | **Significant gap** |
| Wait for agent | Automatic notification | `wait_agent` | Parity |
| Send message to agent | `SendMessage` | `send_input` | Parity |
| Agent worktree isolation | `isolation: "worktree"` param | None | Gap |
| Background agents | `run_in_background` param | None | Gap |
| Agent teams | `TeamCreate`/`TeamDelete` | None | Gap |

### Web & External

| Capability | Claude Code | Orbit Code | Gap |
|-----------|------------|-----------|-----|
| Web search | `WebSearch` | None | Gap |
| Fetch web page | `WebFetch` | None | Gap |
| LSP integration | `LSP` tool | None | Gap |
| MCP tool calls | Via `ToolSearch` + deferred loading | `mcp` handler + `tool_search` | Parity |
| Browser automation | `Computer` tool | MCP browser tools | Parity via MCP |

### User Interaction

| Capability | Claude Code | Orbit Code | Gap |
|-----------|------------|-----------|-----|
| Ask user question | `AskUserQuestion` | `request_user_input` | Parity |
| Request permissions | Via permission mode system | `request_permissions` | Parity |
| Cron/scheduled tasks | `CronCreate`/`CronDelete` | None | Gap |

---

## Experimental Tools Detail

### `grep_files` — 176 lines of Rust

**What it does:** Wraps `rg` (ripgrep) to search file contents by regex pattern, returning matching file paths sorted by modification time.

**Schema:**
```json
{
  "pattern": "regex pattern (required)",
  "include": "glob filter e.g. *.rs (optional)",
  "path": "directory to search (optional, defaults to cwd)",
  "limit": "max results, default 100, max 2000"
}
```

**Implementation highlights:**
- Uses `tokio::process::Command` to run `rg --files-with-matches --sortr=modified`
- 30-second timeout via `tokio::time::timeout`
- Validates path exists before searching
- Handles rg exit codes (0=matches, 1=no matches, other=error)
- Proper error messages if rg is not installed

**Claude Code equivalent:** `Grep` tool — more features (content output, line numbers, context lines, multiline mode, count mode) but same core concept.

### `read_file` — 489 lines of Rust

**What it does:** Reads local files with 1-indexed line numbers. Supports two modes:
1. **Slice mode** (default) — simple offset + limit range read
2. **Indentation mode** — expands around an anchor line following indentation structure

**Schema:**
```json
{
  "file_path": "absolute path (required)",
  "offset": "1-indexed start line (default: 1)",
  "limit": "max lines to return (default: 2000)",
  "mode": "slice | indentation",
  "indentation": {
    "anchor_line": "line to center on",
    "max_levels": "parent indent levels to include",
    "include_siblings": "include same-level blocks",
    "include_header": "include doc comments above block",
    "max_lines": "hard cap on returned lines"
  }
}
```

**Implementation highlights:**
- Line numbering with configurable tab width (4 spaces)
- Long line truncation (500 char max)
- Indentation-aware block extraction (unique feature — Claude Code doesn't have this)
- Comment prefix detection (`#`, `//`, `--`)
- 13K of tests covering edge cases

**Claude Code equivalent:** `Read` tool — simpler (no indentation mode) but supports images, PDFs, Jupyter notebooks.

### `list_dir` — 271 lines of Rust

**What it does:** Lists directory entries with 1-indexed numbering, type labels, and recursive depth traversal.

**Schema:**
```json
{
  "dir_path": "absolute path (required)",
  "offset": "1-indexed start entry (default: 1)",
  "limit": "max entries (default: 25)",
  "depth": "max directory depth (default: 2)"
}
```

**Implementation highlights:**
- Recursive BFS traversal with configurable depth
- Type labels (file, dir, symlink, etc.)
- Indented output for nested directories
- Entry truncation (500 char max)
- Sorted entries within each directory

**Claude Code equivalent:** No dedicated tool — Claude Code uses `Bash("ls")` for directory listing. `Glob` is the closest but serves a different purpose (pattern matching, not listing).

---

## System Prompt Comparison

### Sections Present in Both

| Section | Claude Code File(s) | Orbit Code File |
|---------|--------------------|-----------------|
| Identity/personality | `system-prompt-system-section.md` | `prompt.md` lines 1-15 |
| Task execution | `doing-tasks-software-engineering-focus.md` + 12 sub-files | `prompt.md` "Task execution" section |
| Over-engineering avoidance | `doing-tasks-avoid-over-engineering.md` | `prompt.md` "Avoid unneeded complexity" |
| Git safety | `bash-git-*` (4 files) | `gpt-5.2-codex_prompt.md` "Using GIT" |
| No git commit unless asked | `bash-git-commit-and-pr-creation-instructions.md` | `prompt.md` line 144 |
| Progress updates | Part of agent tool notes | `prompt.md` "Sharing progress updates" |
| Final answer formatting | `tone-and-style-*` + `output-efficiency.md` | `prompt.md` "Final answer structure" (very detailed) |
| Code style guidance | Various `doing-tasks-*` files | `gpt-5.2-codex_prompt.md` "Code style" |
| Validation/testing | Not explicit in system prompt | `prompt.md` "Validating your work" |
| Planning | `tool-description-enterplanmode.md` | `templates/collaboration_mode/plan.md` |
| Context compaction | `system-prompt-context-compaction-summary.md` | `templates/compact/prompt.md` |

### Sections in Claude Code but NOT Orbit Code

| Section | File(s) | Impact |
|---------|---------|--------|
| **Negative tool routing** | Embedded in Bash tool + `tool-usage-*.md` (6 files) | **Critical** — prevents misrouting |
| **Meta tool routing** | `tool-usage-direct-search.md`, `tool-usage-delegate-exploration.md` | **Critical** — decision tree |
| **Memory system** | `system-prompt-agent-memory-instructions.md` + type descriptions | Persistent cross-session knowledge |
| **Hooks system** | `system-prompt-hooks-configuration.md` | Extensible automation |
| **Skills/slash commands** | `tool-description-skill.md` | Extensibility |
| **Executing actions with care** | `system-prompt-executing-actions-with-care.md` | Risk reasoning |
| **Sandbox detailed rules** | 17 `bash-sandbox-*.md` files | Sandbox behavior teaching |
| **Learning mode** | `system-prompt-learning-mode.md` | Educational interaction |
| **Auto mode** | `system-prompt-auto-mode.md` | Autonomous execution |
| **Scratchpad directory** | `system-prompt-scratchpad-directory.md` | Temp file management |
| **Deferred tool loading** | `tool-description-toolsearch-second-part.md` | Cognitive load management |
| **Git commit workflow recipe** | Inside `bash-git-commit-and-pr-creation-instructions.md` | Step-by-step with parallel hints |
| **PR creation workflow recipe** | Same file | Step-by-step with HEREDOC templates |
| **Sleep avoidance rules** | 6 `bash-sleep-*.md` files | Prevents idle loops |
| **CLAUDE.md spec** | Runtime injection | Project-specific instructions |
| **Fork/worktree guidelines** | `system-prompt-fork-usage-guidelines.md` | Isolated development |
| **Subagent prompt writing** | `system-prompt-writing-subagent-prompts.md` | Delegation quality |

### Sections in Orbit Code but NOT Claude Code

| Section | File | Impact |
|---------|------|--------|
| **AGENTS.md spec** | `prompt.md` lines 17-27 | Hierarchical instruction files (OpenAI convention) |
| **Preamble messages** | `prompt.md` "Responsiveness" | 8 examples of pre-tool-call updates |
| **Plan quality examples** | `prompt.md` lines 74-121 | 3 good + 3 bad plan examples |
| **apply_patch grammar** | `prompt_with_apply_patch_instructions.md` | Formal BNF grammar |
| **Ambition vs precision** | `prompt.md` "Ambition vs. precision" | Creative vs surgical philosophy |
| **Frontend design rules** | `gpt-5.2-codex_prompt.md` "Frontend tasks" | Anti-AI-slop guidelines |
| **Review mode** | `gpt-5.2-codex_prompt.md` "Reviews" | Code review mindset |
| **Collaboration modes** | `templates/collaboration_mode/` (4 modes) | Default/execute/pair/plan |
| **Orchestrator template** | `templates/agents/orchestrator.md` | Sub-agent coordination |
| **Guardian policy** | `src/guardian/policy.md` | AI-based risk assessment |
| **Personality presets** | `templates/personalities/` | Pragmatic/friendly |
| **Code mode (exec)** | `code_mode/description.md` | JS REPL with persistent state |
| **Model-specific prompts** | `gpt_5_1_prompt.md`, `gpt_5_2_prompt.md`, etc. | Per-model optimization |
| **Verbosity tiers** | `gpt_5_1_prompt.md` "Verbosity" section | Tiny/small/medium/large rules |
| **User Updates Spec** | `gpt_5_1_prompt.md` "User Updates Spec" | Frequency/length/content rules |
| **Autonomy persistence** | `gpt_5_1_prompt.md` "Autonomy and Persistence" | End-to-end completion |

---

## Tool Description Patterns

### Claude Code Pattern (behavioral instructions)

Claude Code tool descriptions are **mini-manuals**, not API docs. Example from the `Bash` tool (~2,000 tokens):

```
Executes a given bash command and returns its output.

IMPORTANT: Avoid using this tool to run find, grep, cat, head, tail,
sed, awk, or echo commands. Instead, use the appropriate dedicated tool:
 - File search: Use Glob (NOT find or ls)
 - Content search: Use Grep (NOT grep or rg)
 - Read files: Use Read (NOT cat/head/tail)
 ...

# Instructions
 - If your command will create new directories, first verify parent exists
 - Always quote file paths with spaces
 - Try to maintain your current working directory
 ...

# Committing changes with git
[40+ lines of step-by-step workflow with parallel execution hints]

# Creating pull requests
[30+ lines of step-by-step workflow with HEREDOC templates]
```

Key patterns:
1. **Negative routing** — "Do NOT use X for Y, use Z instead"
2. **Workflow recipes** — Step-by-step instructions embedded in tool descriptions
3. **Error recovery** — "The edit will FAIL if old_string is not unique. Either provide more context..."
4. **Dependencies** — "You must use Read at least once before editing"
5. **Parallel hints** — "Run the following bash commands in parallel"

### Orbit Code Pattern (API reference)

Orbit Code tool descriptions are **minimal**:

```
grep_files: "Finds files whose contents match the pattern and lists them
            by modification time."

read_file: "Reads a local file with 1-indexed line numbers, supporting
           slice and indentation-aware block modes."

list_dir: "Lists entries in a local directory with 1-indexed entry numbers
          and simple type labels."
```

No behavioral instructions, no routing guidance, no workflow recipes.

---

## Key Prompt Engineering Patterns Claude Code Uses

### 1. Negative Routing (appears 3x)

The rule "don't use shell for file reads" appears in:
- System prompt `# Using your tools` section
- Bash tool `IMPORTANT:` block
- Each dedicated tool's own description

### 2. Meta-Routing Decision Tree

A `# Using your tools` section tells the model HOW to pick tools:
```
- Simple, directed searches → Glob or Grep directly
- Broader exploration → Agent with Explore subagent
- Slash commands → Skill tool
- Complex multi-step → Agent tool
```

### 3. Deferred Loading

Only ~8 tools loaded upfront. Others listed by name in `<available-deferred-tools>`. Model calls `ToolSearch` to fetch schemas when needed. Reduces cognitive load.

### 4. Typed Subagents

Agent tool spawns specialized subagents with restricted tool access:
- `Explore` → read-only (Glob, Grep, LS, Read)
- `Plan` → read-only
- `general-purpose` → all tools
- `code-reviewer` → read + analysis

### 5. Redundancy by Design

Same concept stated in multiple locations to ensure the model attends to it regardless of which prompt section is in the attention window.

### 6. Workflow Templates

Complete recipes for git commit, PR creation, and other multi-step workflows embedded directly in tool descriptions with parallel execution hints.

### 7. Per-Tool Error Recovery

Each tool description explains what happens when it fails and how to recover, teaching the model to reason about failures rather than retry blindly.

---

## Migration Recommendations

### Phase 1: Quick Wins (1-3 days)

1. **Enable `read_file`, `grep_files`, `list_dir`** — Remove experimental gate in `spec.rs` or populate `experimental_supported_tools` in `models.json`. These are tested, production-ready handlers.

2. **Add negative routing to shell tool description** — In the system prompt, add:
   ```
   When read_file is available, use it instead of cat/head/tail.
   When grep_files is available, use it instead of grep/rg.
   When list_dir is available, use it instead of ls.
   ```

3. **Add meta-routing section to system prompt** — Add a `# Using your tools` section that tells the model when to use which tool.

### Phase 2: New Tools (1-2 weeks)

4. **Build `Glob` handler** — ~150 lines. Wrap the `glob` crate. Parameters: `pattern`, `path`. Returns file paths sorted by modification time.

5. **Build `Edit` handler** — Exact string find-and-replace. Simpler than `apply_patch` but more reliable for small edits. Parameters: `file_path`, `old_string`, `new_string`, `replace_all`.

6. **Build `Write` handler** — Direct file write. Parameters: `file_path`, `content`. Simpler than `apply_patch` for new file creation.

### Phase 3: Prompt Engineering (1-2 weeks)

7. **Rewrite tool descriptions** — Transform from API-reference style to Claude Code's behavioral-instruction style. Each description should include:
   - What the tool does
   - When to use it (and when NOT to)
   - Workflow patterns
   - Error recovery guidance
   - Dependencies on other tools

8. **Add redundancy** — Key routing rules should appear in:
   - System prompt `# Using your tools`
   - Shell tool description
   - Each dedicated tool's description

9. **Add workflow recipes** — Embed git commit and PR creation workflows in the shell tool description.

### Phase 4: Advanced (2-4 weeks)

10. **Typed subagents** — Add subagent type system to `spawn_agent` with tool restrictions per type.

11. **Deferred tool loading** — Only load core tools upfront, defer others. Requires changes to `ToolRouter` and `spec.rs`.

12. **Memory system** — Persistent cross-session memory (user preferences, project context, feedback).

### What NOT to build

- **WebSearch/WebFetch** — Not needed if targeting local development workflows
- **LSP** — Nice to have but very complex; MCP servers can provide this
- **CronCreate** — Niche feature, low priority
- **NotebookEdit** — Unless Jupyter is a core use case
- **TeamCreate/TeamDelete** — Unless building multi-agent swarm support

---

## Appendix: File Locations

### Orbit Code Key Files

| File | Purpose |
|------|---------|
| `codex-rs/core/src/tools/handlers/*.rs` | All tool handler implementations |
| `codex-rs/core/src/tools/spec.rs` | Tool spec creation + registration logic |
| `codex-rs/core/src/tools/registry.rs` | `ToolHandler` trait + `ToolRegistry` |
| `codex-rs/core/src/tools/router.rs` | `ToolRouter` — builds and dispatches tool calls |
| `codex-rs/core/models.json` | Model definitions including `experimental_supported_tools` |
| `codex-rs/core/prompt.md` | Base system prompt |
| `codex-rs/core/gpt_5_1_prompt.md` | GPT-5.1 system prompt (most detailed) |
| `codex-rs/core/gpt-5.2-codex_prompt.md` | GPT-5.2 Codex system prompt |
| `codex-rs/core/templates/collaboration_mode/` | Collaboration mode presets |
| `codex-rs/core/templates/agents/orchestrator.md` | Multi-agent orchestrator prompt |

### Claude Code Reference (Piebald repo)

| Directory | Content |
|-----------|---------|
| `reference/claude-code-system-prompts/system-prompts/` | 247 extracted prompt files |
| `reference/claude-code-system-prompts/tools/updatePrompts.js` | Extraction pipeline script |
| `reference/claude-code-system-prompts/CHANGELOG.md` | Prompt changes across 130+ versions |
| `reference/claude-code-system-prompts/README.md` | Categorized index with token counts |

### File Categories in Piebald Repo

| Prefix | Count | Content |
|--------|------:|---------|
| `system-prompt-*` | ~50 | Core system prompt sections |
| `tool-description-*` | ~70 | Tool descriptions (including ~30 Bash sub-sections) |
| `agent-prompt-*` | ~35 | Subagent system prompts |
| `data-*` | ~25 | Reference data (API docs, SDK patterns) |
| `skill-*` | ~15 | Built-in skill definitions |
| `system-reminder-*` | ~40 | Contextual system reminders |
