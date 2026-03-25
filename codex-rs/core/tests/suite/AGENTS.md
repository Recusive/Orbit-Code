# codex-rs/core/tests/suite/

This file applies to `codex-rs/core/tests/suite/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Comprehensive integration test modules for `codex-core`.

### What this folder does

Contains all integration test modules, aggregated into a single test binary via `all.rs -> mod suite`. Each file tests a specific feature area of `codex-core`.

### Test setup

The `mod.rs` file sets up test infrastructure:
- Creates `CODEX_ALIASES_TEMP_DIR` via `ctor` for arg0 dispatch (apply_patch, codex-linux-sandbox)
- Manages `ORBIT_HOME` environment variable for isolated test environments

### Test modules (selected highlights)

| Module | Tests |
|--------|-------|
| `tools.rs` | Tool registration, routing, and execution |
| `shell_command.rs` | Shell command execution and sandboxing |
| `unified_exec.rs` | Interactive process management |
| `exec.rs` | One-shot command execution |
| `apply_patch_cli.rs` | File patch application via CLI |
| `approvals.rs` | Approval flow (auto-approve, manual, guardian) |
| `compact.rs` / `compact_remote.rs` | Context compaction |
| `resume.rs` / `fork_thread.rs` | Session resume and forking |
| `hierarchical_agents.rs` | Multi-agent spawning and coordination |
| `skills.rs` / `skill_approval.rs` | Skill loading and execution |
| `plugins.rs` | Plugin management |
| `memories.rs` | Memory extraction and consolidation |
| `seatbelt.rs` | macOS sandbox integration |
| `model_visible_layout.rs` | Verifying what the model sees in context |
| `hooks.rs` | Hook execution |
| `auth_refresh.rs` | OAuth token refresh |
| `rollout_list_find.rs` | Session listing and discovery |
| `otel.rs` | OpenTelemetry metric emission |
| `web_search.rs` | Web search tool |
| `personality.rs` | Personality configuration |
| `live_reload.rs` | Config live reload |

### Where it plugs into

- Compiled as part of `codex-core`'s integration test binary
- Uses `core_test_support` from `tests/common/` for shared utilities
- Snapshot tests store expected outputs in `snapshots/`
