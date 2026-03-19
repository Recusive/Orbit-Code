# Stage 2: Remove Dead Crates

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove 8 OpenAI-only crates that will never be used in Orbit Code. Clean all references from the workspace, CLI dispatcher, app-server, exec, and TUI crates so the project compiles and tests pass without them.

**Architecture:** Bottom-up removal — delete leaf crates first (no dependents), then work up the dependency chain. The `cloud-requirements` crate requires stub replacement since 4 crates depend on its `CloudRequirementsLoader` interface.

**Tech Stack:** Rust (Cargo workspace), file deletion, targeted source edits.

**Depends on:** Stage 1 (rename) must be complete. Crate names below use post-rename names (`orbit-code-*`).

---

## Crates to Remove

| # | Crate (post-rename) | Path | Direct Dependents | Removal Complexity |
|---|---------------------|------|-------------------|-------------------|
| 1 | `orbit-code-backend-openapi-models` | `codex-rs/codex-backend-openapi-models/` | backend-client | EASY — leaf |
| 2 | `orbit-code-connectors` | `codex-rs/connectors/` | chatgpt | EASY — leaf |
| 3 | `orbit-code-cloud-tasks-client` | `codex-rs/cloud-tasks-client/` | cloud-tasks | EASY — leaf |
| 4 | `orbit-code-chatgpt` | `codex-rs/chatgpt/` | cli, app-server | MEDIUM — remove imports + CLI subcommand |
| 5 | `orbit-code-backend-client` | `codex-rs/backend-client/` | cloud-tasks-client, cloud-requirements | EASY — dependents already removed |
| 6 | `orbit-code-cloud-tasks` | `codex-rs/cloud-tasks/` | cli | MEDIUM — remove CLI subcommand |
| 7 | `orbit-code-responses-api-proxy` | `codex-rs/responses-api-proxy/` | cli | MEDIUM — remove CLI subcommand + npm package |
| 8 | `orbit-code-cloud-requirements` | `codex-rs/cloud-requirements/` | exec, app-server, tui, tui_app_server | HARDEST — 4 dependents need stub/removal of CloudRequirementsLoader |

## Dependency Graph (removal order)

```
Remove first (leaves):
  backend-openapi-models ──→ (only used by backend-client)
  connectors ──→ (only used by chatgpt)
  cloud-tasks-client ──→ (only used by cloud-tasks)

Remove second (mid-level):
  chatgpt ──→ (used by cli, app-server)
  backend-client ──→ (dependents already gone)
  cloud-tasks ──→ (used by cli)
  responses-api-proxy ──→ (used by cli)

Remove last (most dependents):
  cloud-requirements ──→ (used by exec, app-server, tui, tui_app_server)
```

---

## Files Modified

**Workspace root:**
- `codex-rs/Cargo.toml` — remove members + workspace.dependencies entries

**CLI crate (remove subcommands):**
- `codex-rs/cli/Cargo.toml` — remove 3 dependencies
- `codex-rs/cli/src/main.rs` — remove `apply`, `cloud`, `responses-api-proxy` subcommands and imports

**App-server (remove connectors + cloud-requirements):**
- `codex-rs/app-server/Cargo.toml` — remove 2 dependencies
- `codex-rs/app-server/src/lib.rs` — remove cloud_requirements_loader usage
- `codex-rs/app-server/src/codex_message_processor.rs` — remove connectors import + related functions

**Exec (remove cloud-requirements):**
- `codex-rs/exec/Cargo.toml` — remove 1 dependency
- `codex-rs/exec/src/lib.rs` — remove cloud_requirements_loader import + usage

**TUI (remove cloud-requirements):**
- `codex-rs/tui/Cargo.toml` — remove 1 dependency

**TUI App Server (remove cloud-requirements):**
- `codex-rs/tui_app_server/Cargo.toml` — remove 1 dependency
- `codex-rs/tui_app_server/src/lib.rs` — remove cloud_requirements_loader_for_storage import + usage

**Directories deleted:**
- `codex-rs/chatgpt/`
- `codex-rs/backend-client/`
- `codex-rs/codex-backend-openapi-models/`
- `codex-rs/connectors/`
- `codex-rs/responses-api-proxy/`
- `codex-rs/cloud-requirements/`
- `codex-rs/cloud-tasks/`
- `codex-rs/cloud-tasks-client/`

---

### Task 1: Remove Leaf Crates (No Dependents Outside Dead Set)

**Files:**
- Delete: `codex-rs/codex-backend-openapi-models/` (entire directory)
- Delete: `codex-rs/connectors/` (entire directory)
- Delete: `codex-rs/cloud-tasks-client/` (entire directory)
- Modify: `codex-rs/Cargo.toml` — remove from members + workspace.dependencies

- [ ] **Step 1: Delete the three leaf directories**

```bash
rm -rf codex-rs/codex-backend-openapi-models/
rm -rf codex-rs/connectors/
rm -rf codex-rs/cloud-tasks-client/
```

- [ ] **Step 2: Remove from workspace Cargo.toml members array**

Remove these entries from the `[workspace] members` array in `codex-rs/Cargo.toml`:
- `"codex-backend-openapi-models"`
- `"connectors"`
- `"cloud-tasks-client"`

- [ ] **Step 3: Remove from workspace.dependencies**

Remove these lines from `[workspace.dependencies]` in `codex-rs/Cargo.toml`:
- `orbit-code-connectors = { path = "connectors" }` (was `codex-connectors`)

(Note: `backend-openapi-models` and `cloud-tasks-client` may not have workspace.dependencies entries — check and remove if present)

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "Remove leaf crates: backend-openapi-models, connectors, cloud-tasks-client"
```

---

### Task 2: Remove chatgpt, backend-client, cloud-tasks, responses-api-proxy

**Files:**
- Delete: `codex-rs/chatgpt/`, `codex-rs/backend-client/`, `codex-rs/cloud-tasks/`, `codex-rs/responses-api-proxy/`
- Modify: `codex-rs/Cargo.toml` — remove members + deps
- Modify: `codex-rs/cli/Cargo.toml` — remove 3 dependencies
- Modify: `codex-rs/cli/src/main.rs` — remove subcommands and imports
- Modify: `codex-rs/app-server/Cargo.toml` — remove chatgpt dependency
- Modify: `codex-rs/app-server/src/codex_message_processor.rs` — remove connectors usage

- [ ] **Step 1: Delete the four directories**

```bash
rm -rf codex-rs/chatgpt/
rm -rf codex-rs/backend-client/
rm -rf codex-rs/cloud-tasks/
rm -rf codex-rs/responses-api-proxy/
```

- [ ] **Step 2: Remove from workspace Cargo.toml**

Remove from members:
- `"chatgpt"`
- `"backend-client"`
- `"cloud-tasks"`
- `"responses-api-proxy"`

Remove from workspace.dependencies:
- `orbit-code-chatgpt = { path = "chatgpt" }`
- `orbit-code-backend-client = { path = "backend-client" }`
- `orbit-code-responses-api-proxy = { path = "responses-api-proxy" }`

- [ ] **Step 3: Remove from CLI crate**

In `codex-rs/cli/Cargo.toml`, remove:
```toml
orbit-code-chatgpt = { workspace = true }
orbit-code-cloud-tasks = { path = "../cloud-tasks" }
orbit-code-responses-api-proxy = { workspace = true }
```

In `codex-rs/cli/src/main.rs`:
- Remove `use orbit_code_chatgpt::apply_command::ApplyCommand;`
- Remove `use orbit_code_chatgpt::apply_command::run_apply_command;`
- Remove `use orbit_code_cloud_tasks::Cli as CloudTasksCli;`
- Remove `use orbit_code_responses_api_proxy::Args as ResponsesApiProxyArgs;`
- Remove the `apply` subcommand match arm and its handler
- Remove the `cloud` / `cloud-tasks` subcommand match arm and its handler
- Remove the `responses-api-proxy` subcommand match arm and its handler

- [ ] **Step 4: Remove chatgpt from app-server**

In `codex-rs/app-server/Cargo.toml`, remove:
```toml
orbit-code-chatgpt = { workspace = true }
```

In `codex-rs/app-server/src/codex_message_processor.rs`:
- Remove `use orbit_code_chatgpt::connectors;`
- Remove or stub out any functions that call `connectors::*`
- For `list_apps` / `connectors_for_plugin_apps`: return empty vec instead

- [ ] **Step 5: cargo check**

Run: `cd codex-rs && cargo check 2>&1 | head -50`
Expected: Errors only from cloud-requirements (Task 3)

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "Remove chatgpt, backend-client, cloud-tasks, responses-api-proxy crates and CLI subcommands"
```

---

### Task 3: Remove cloud-requirements (Most Dependents)

This crate provides `CloudRequirementsLoader` used by exec, app-server, tui, and tui_app_server. We replace calls with `None` (no cloud requirements for Orbit Code — we'll build our own enterprise config system later if needed).

**Files:**
- Delete: `codex-rs/cloud-requirements/`
- Modify: `codex-rs/Cargo.toml` — remove member + dep
- Modify: `codex-rs/exec/Cargo.toml` + `src/lib.rs`
- Modify: `codex-rs/app-server/Cargo.toml` + `src/lib.rs`
- Modify: `codex-rs/tui/Cargo.toml`
- Modify: `codex-rs/tui_app_server/Cargo.toml` + `src/lib.rs`

- [ ] **Step 1: Delete the directory**

```bash
rm -rf codex-rs/cloud-requirements/
```

- [ ] **Step 2: Remove from workspace Cargo.toml**

Remove from members: `"cloud-requirements"`
Remove from deps: `orbit-code-cloud-requirements = { path = "cloud-requirements" }`

- [ ] **Step 3: Update exec crate**

In `codex-rs/exec/Cargo.toml`, remove:
```toml
orbit-code-cloud-requirements = { workspace = true }
```

In `codex-rs/exec/src/lib.rs`:
- Remove `use orbit_code_cloud_requirements::cloud_requirements_loader;`
- Where `cloud_requirements_loader(...)` is called, replace with `None` (the parameter that accepts the loader is `Option<CloudRequirementsLoader>`)

- [ ] **Step 4: Update app-server crate**

In `codex-rs/app-server/Cargo.toml`, remove:
```toml
orbit-code-cloud-requirements = { workspace = true }
```

In `codex-rs/app-server/src/lib.rs`:
- Remove `use orbit_code_cloud_requirements::cloud_requirements_loader;`
- Replace `cloud_requirements_loader(...)` call with `None`

In `codex-rs/app-server/src/codex_message_processor.rs`:
- Remove `use orbit_code_cloud_requirements::cloud_requirements_loader;`
- Replace any `cloud_requirements_loader(...)` call with `None`

- [ ] **Step 5: Update tui crate**

In `codex-rs/tui/Cargo.toml`, remove:
```toml
orbit-code-cloud-requirements = { workspace = true }
```

(The TUI may not directly call the loader — check if there are source imports too)

- [ ] **Step 6: Update tui_app_server crate**

In `codex-rs/tui_app_server/Cargo.toml`, remove:
```toml
orbit-code-cloud-requirements = { workspace = true }
```

In `codex-rs/tui_app_server/src/lib.rs`:
- Remove `use orbit_code_cloud_requirements::cloud_requirements_loader_for_storage;`
- Replace both `cloud_requirements_loader_for_storage(...)` calls with `None`

- [ ] **Step 7: Check if CloudRequirementsLoader type is defined in core**

The `CloudRequirementsLoader` type may be defined in `orbit-code-core::config_loader`. If so, it stays (it's a trait/type that accepts `Option`). We just stop providing an implementation.

Run: `grep -rn 'CloudRequirementsLoader' codex-rs/core/src/ | head -10`

If it's a type alias or trait in core, leave it. The callers now pass `None` instead of a real loader.

- [ ] **Step 8: cargo check**

Run: `cd codex-rs && cargo check`
Expected: Clean compilation

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "Remove cloud-requirements crate — pass None for cloud config loader"
```

---

### Task 4: Clean Up npm Package for responses-api-proxy

**Files:**
- Delete: `codex-rs/responses-api-proxy/npm/` (if not already deleted with parent)
- Modify: `pnpm-workspace.yaml` — remove responses-api-proxy/npm entry

- [ ] **Step 1: Remove from pnpm workspace**

In `pnpm-workspace.yaml`, remove:
```yaml
  - codex-rs/responses-api-proxy/npm
```

- [ ] **Step 2: Commit**

```bash
git add -A
git commit -m "Remove responses-api-proxy npm package from workspace"
```

---

### Task 5: Regenerate Cargo.lock and Verify

- [ ] **Step 1: Regenerate Cargo.lock**

Run: `cd codex-rs && cargo generate-lockfile`

- [ ] **Step 2: Full build**

Run: `cd codex-rs && cargo build 2>&1 | tail -20`
Expected: Clean build

- [ ] **Step 3: Run tests**

Run: `cd codex-rs && cargo test -p orbit-code-core 2>&1 | tail -20`
Then: `cd codex-rs && cargo test -p orbit-code-cli 2>&1 | tail -20`
Expected: Pass (no tests should reference removed crates)

- [ ] **Step 4: Verify removed crates are gone**

```bash
for crate in chatgpt backend-client codex-backend-openapi-models connectors responses-api-proxy cloud-requirements cloud-tasks cloud-tasks-client; do
  if [ -d "codex-rs/$crate" ]; then
    echo "STILL EXISTS: $crate"
  fi
done
```
Expected: No output

- [ ] **Step 5: Verify no imports of removed crates**

```bash
grep -rn 'orbit_code_chatgpt\|orbit_code_backend_client\|orbit_code_cloud_tasks\|orbit_code_cloud_requirements\|orbit_code_connectors\|orbit_code_responses_api_proxy\|orbit_code_backend_openapi' \
  codex-rs/ --include='*.rs' --include='*.toml' | grep -v target/ | head -10
```
Expected: No output

- [ ] **Step 6: Commit and push**

```bash
git add -A
git commit -m "Stage 2 complete: removed 8 OpenAI-only crates, clean build"
git push origin main
```

---

## Post-Removal: What's Left

After removing 8 crates, the workspace has ~65 crates remaining, all generic infrastructure:

**Core engine:** core, protocol, config, state
**TUI:** tui, tui_app_server
**Servers:** app-server, app-server-protocol, app-server-client, mcp-server, exec-server
**Execution:** exec, cli, arg0
**API client (to be extended in Stage 3):** codex-api (now orbit-code-api), codex-client
**Sandbox:** linux-sandbox, windows-sandbox-rs, execpolicy, execpolicy-legacy
**Auth:** login, keyring-store, secrets
**Tools:** apply-patch, shell-command, shell-escalation, file-search
**Hooks/Skills:** hooks, skills
**Utils:** 19 utility crates
**Local models:** lmstudio, ollama
**Other:** ansi-escape, artifacts, async-utils, debug-client, environment, feedback, otel, network-proxy, package-manager, process-hardening, rmcp-client, stdio-to-uds, test-macros

~15,000 lines of OpenAI-specific code removed. The codebase is now a clean foundation for multi-provider support in Stage 3.
