# Stage 2: Remove Dead Crates

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove 8 OpenAI-only crates that will never be used in Orbit Code. Clean all references from the workspace, CLI dispatcher, app-server, exec, core, and both TUI crates so the project compiles and tests pass without them.

**Architecture:** Bottom-up removal — delete leaf crates first (no dependents), then work up the dependency chain. The `connectors` crate requires stub/inline replacement in `core`. The `cloud-requirements` crate requires stub replacement in 4 crates. The `chatgpt` and `backend-client` crates have imports across 6 crates that need updating.

**Tech Stack:** Rust (Cargo workspace), file deletion, targeted source edits.

**Depends on:** Stage 1 (rename) must be complete. Crate names below use post-rename names (`orbit-code-*`).

---

## Crates to Remove

| # | Crate (post-rename) | Path | Direct Dependents | Removal Complexity |
|---|---------------------|------|-------------------|-------------------|
| 1 | `orbit-code-backend-openapi-models` | `codex-rs/codex-backend-openapi-models/` | backend-client | EASY — leaf |
| 2 | `orbit-code-cloud-tasks-client` | `codex-rs/cloud-tasks-client/` | cloud-tasks | EASY — leaf |
| 3 | `orbit-code-connectors` | `codex-rs/connectors/` | chatgpt, **core** | MEDIUM — core uses `AllConnectorsCacheKey`, `DirectoryListResponse`, `CONNECTORS_CACHE_TTL`, `list_all_connectors_with_options()` |
| 4 | `orbit-code-chatgpt` | `codex-rs/chatgpt/` | cli, app-server, **tui, tui_app_server** | HARD — `connectors::AppInfo` type used in chatwidget, skills, composer, app_event + extensive tests |
| 5 | `orbit-code-backend-client` | `codex-rs/backend-client/` | cloud-tasks-client, cloud-requirements, **tui, app-server** | MEDIUM — tui + app-server use `BackendClient::from_auth()` for rate limits |
| 6 | `orbit-code-cloud-tasks` | `codex-rs/cloud-tasks/` | cli | MEDIUM — remove CLI subcommand |
| 7 | `orbit-code-responses-api-proxy` | `codex-rs/responses-api-proxy/` | cli | MEDIUM — remove CLI subcommand + npm package |
| 8 | `orbit-code-cloud-requirements` | `codex-rs/cloud-requirements/` | exec, app-server, tui, tui_app_server | HARDEST — 4 dependents need stub/removal of CloudRequirementsLoader |

## Dependency Graph (removal order)

```
Remove first (true leaves — no live dependents outside dead set):
  backend-openapi-models ──→ (only used by backend-client, which is also being removed)
  cloud-tasks-client ──→ (only used by cloud-tasks, which is also being removed)

Remove second (mid-level — dependents need source edits):
  connectors ──→ (used by chatgpt [dead] + core [LIVE — needs stub])
  chatgpt ──→ (used by cli, app-server, tui, tui_app_server)
  backend-client ──→ (used by cloud-tasks-client [dead], cloud-requirements [dead], tui [LIVE], app-server [LIVE])
  cloud-tasks ──→ (used by cli)
  responses-api-proxy ──→ (used by cli)

Remove last (most dependents):
  cloud-requirements ──→ (used by exec, app-server, tui, tui_app_server)
```

---

## Files Modified

**Workspace root:**
- `codex-rs/Cargo.toml` — remove members + workspace.dependencies entries

**Core crate (remove connectors dependency):**
- `codex-rs/core/Cargo.toml` — remove 1 dependency (`orbit-code-connectors`)
- `codex-rs/core/src/connectors.rs` — replace `orbit_code_connectors` imports with inline stubs/returns

**CLI crate (remove subcommands):**
- `codex-rs/cli/Cargo.toml` — remove 3 dependencies
- `codex-rs/cli/src/main.rs` — remove `apply`, `cloud`, `responses-api-proxy` subcommands and imports

**App-server (remove chatgpt, backend-client, connectors, cloud-requirements):**
- `codex-rs/app-server/Cargo.toml` — remove 4 dependencies
- `codex-rs/app-server/src/lib.rs` — remove cloud_requirements_loader usage
- `codex-rs/app-server/src/orbit_code_message_processor.rs` — remove `BackendClient`, `connectors`, `cloud_requirements_loader` imports; stub rate limits + connectors

**Exec (remove cloud-requirements):**
- `codex-rs/exec/Cargo.toml` — remove 1 dependency
- `codex-rs/exec/src/lib.rs` — remove cloud_requirements_loader import + usage

**TUI (remove chatgpt, backend-client, cloud-requirements):**
- `codex-rs/tui/Cargo.toml` — remove 3 dependencies
- `codex-rs/tui/src/lib.rs` — remove `cloud_requirements_loader` import + all 15+ usages; replace with `CloudRequirementsLoader::default()`
- `codex-rs/tui/src/chatwidget.rs` — replace `orbit_code_backend_client::Client` and `orbit_code_chatgpt::connectors` imports
- `codex-rs/tui/src/chatwidget/skills.rs` — replace `orbit_code_chatgpt::connectors::AppInfo` import
- `codex-rs/tui/src/bottom_pane/chat_composer.rs` — replace `orbit_code_chatgpt::connectors` imports
- `codex-rs/tui/src/app_event.rs` — replace `orbit_code_chatgpt::connectors::AppInfo` import
- `codex-rs/tui/src/chatwidget/tests.rs` — replace all `orbit_code_chatgpt::connectors::AppInfo` usages

**TUI App Server (remove chatgpt, cloud-requirements):**
- `codex-rs/tui_app_server/Cargo.toml` — remove 2 dependencies
- `codex-rs/tui_app_server/src/lib.rs` — remove `cloud_requirements_loader_for_storage` import + all usages; replace with `CloudRequirementsLoader::default()`
- `codex-rs/tui_app_server/src/chatwidget.rs` — replace `orbit_code_chatgpt::connectors` import
- `codex-rs/tui_app_server/src/chatwidget/skills.rs` — replace `orbit_code_chatgpt::connectors::AppInfo` import
- `codex-rs/tui_app_server/src/bottom_pane/chat_composer.rs` — replace `orbit_code_chatgpt::connectors` imports
- `codex-rs/tui_app_server/src/app_event.rs` — replace `orbit_code_chatgpt::connectors::AppInfo` import
- `codex-rs/tui_app_server/src/chatwidget/tests.rs` — replace all `orbit_code_chatgpt::connectors::AppInfo` usages

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

### Task 1: Remove Leaf Crates (No Live Dependents)

Only `backend-openapi-models` and `cloud-tasks-client` are true leaves — their only dependents are other crates in the dead set.

**Files:**
- Delete: `codex-rs/codex-backend-openapi-models/` (entire directory)
- Delete: `codex-rs/cloud-tasks-client/` (entire directory)
- Modify: `codex-rs/Cargo.toml` — remove from members + workspace.dependencies

- [ ] **Step 1: Delete the two leaf directories**

```bash
rm -rf codex-rs/codex-backend-openapi-models/
rm -rf codex-rs/cloud-tasks-client/
```

- [ ] **Step 2: Remove from workspace Cargo.toml members array**

Remove these entries from the `[workspace] members` array in `codex-rs/Cargo.toml`:
- `"codex-backend-openapi-models"`
- `"cloud-tasks-client"`

- [ ] **Step 3: Remove from workspace.dependencies**

Check `[workspace.dependencies]` in `codex-rs/Cargo.toml` and remove any entries for these crates if present. (They may not have workspace.dependencies entries — verify before editing.)

- [ ] **Step 4: cargo check**

Run: `cd codex-rs && cargo check 2>&1 | head -50`
Expected: Clean (these are true leaves with no live dependents)

- [ ] **Step 5: Commit**

```bash
git add codex-rs/Cargo.toml codex-rs/Cargo.lock
git commit -m "Remove leaf crates: backend-openapi-models, cloud-tasks-client"
```

---

### Task 2: Remove connectors, chatgpt, backend-client, cloud-tasks, responses-api-proxy

These crates have live dependents that need source code edits. The key type migration is:
- `orbit_code_chatgpt::connectors::AppInfo` → `orbit_code_core::connectors::AppInfo` (which re-exports from `orbit_code_app_server_protocol::AppInfo`)
- `orbit_code_chatgpt::connectors` module → `orbit_code_core::connectors` module
- `orbit_code_backend_client::Client` → stub rate limit functions to return empty vecs
- `orbit_code_connectors::*` in core → inline stubs

**Files:**
- Delete: `codex-rs/connectors/`, `codex-rs/chatgpt/`, `codex-rs/backend-client/`, `codex-rs/cloud-tasks/`, `codex-rs/responses-api-proxy/`
- Modify: `codex-rs/Cargo.toml` — remove members + deps
- Modify: `codex-rs/core/Cargo.toml` + `src/connectors.rs` — remove connectors dependency, inline stubs
- Modify: `codex-rs/cli/Cargo.toml` + `src/main.rs` — remove 3 dependencies + subcommands
- Modify: `codex-rs/app-server/Cargo.toml` + `src/orbit_code_message_processor.rs` — remove chatgpt + backend-client deps + imports
- Modify: `codex-rs/tui/Cargo.toml` + multiple source files — remove chatgpt + backend-client deps + imports
- Modify: `codex-rs/tui_app_server/Cargo.toml` + multiple source files — remove chatgpt dep + imports

- [ ] **Step 1: Delete the five directories**

```bash
rm -rf codex-rs/connectors/
rm -rf codex-rs/chatgpt/
rm -rf codex-rs/backend-client/
rm -rf codex-rs/cloud-tasks/
rm -rf codex-rs/responses-api-proxy/
```

- [ ] **Step 2: Remove from workspace Cargo.toml**

Remove from members:
- `"connectors"`
- `"backend-client"`
- `"cloud-tasks"`
- `"responses-api-proxy"`

(Note: `chatgpt` is not in members — it was a library crate referenced only via workspace.dependencies. Check and remove if present.)

Remove from workspace.dependencies:
- `orbit-code-connectors = { path = "connectors" }`
- `orbit-code-chatgpt = { path = "chatgpt" }`
- `orbit-code-backend-client = { path = "backend-client" }`
- `orbit-code-responses-api-proxy = { path = "responses-api-proxy" }`

(Note: `orbit-code-cloud-tasks` is NOT in workspace.dependencies — the CLI references it via a direct path dep `{ path = "../cloud-tasks" }`. No workspace.dependencies entry to remove.)

- [ ] **Step 3: Update core crate (remove connectors dependency)**

In `codex-rs/core/Cargo.toml`, remove:
```toml
orbit-code-connectors = { workspace = true }
```

In `codex-rs/core/src/connectors.rs`:
- Remove `use orbit_code_connectors::AllConnectorsCacheKey;`
- Remove `use orbit_code_connectors::DirectoryListResponse;`
- Remove `pub use orbit_code_connectors::CONNECTORS_CACHE_TTL;`
- Replace with inline stubs:

```rust
// Inline after removing orbit-code-connectors crate
pub const CONNECTORS_CACHE_TTL: Duration = Duration::from_secs(300);
```

- For `AllConnectorsCacheKey` and `DirectoryListResponse`: check the connectors crate source to determine the minimal struct definitions needed, then inline them into `core/src/connectors.rs`. These are used by the `list_all_connectors_with_options()` call at line ~435.

- The `orbit_code_connectors::list_all_connectors_with_options()` call site needs to be replaced. This function fetches connector data from the ChatGPT backend API. Replace with a stub that returns an empty result (no connectors available).

- Also check `codex-rs/core/src/connectors_tests.rs` for any `orbit_code_connectors` imports and update.

- [ ] **Step 4: Update CLI crate**

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
- Remove the `Apply` variant from the `Subcommand` enum and its match arm + handler
- Remove the `Cloud` variant from the `Subcommand` enum and its match arm + handler
- Remove the `ResponsesApiProxy` variant from the `Subcommand` enum and its match arm + handler

- [ ] **Step 5: Update app-server crate (remove chatgpt + backend-client)**

In `codex-rs/app-server/Cargo.toml`, remove:
```toml
orbit-code-chatgpt = { workspace = true }
orbit-code-backend-client = { workspace = true }
```

In `codex-rs/app-server/src/orbit_code_message_processor.rs`:
- Remove `use orbit_code_backend_client::Client as BackendClient;`
- Remove `use orbit_code_chatgpt::connectors;`
- Replace `connectors::*` calls with equivalents from `orbit_code_core::connectors` (the core module re-exports `AppInfo` and provides connector listing functions)
- Stub rate-limit functions that use `BackendClient::from_auth()` to return empty results:

```rust
// Rate limits require OpenAI backend — stub until Orbit backend supports this
async fn get_rate_limits(...) -> ... {
    Ok(Vec::new())
}
```

Also check `codex-rs/app-server/src/orbit_code_message_processor/apps_list_helpers.rs` and `plugin_app_helpers.rs` for chatgpt/backend-client imports.

- [ ] **Step 6: Update TUI crate (remove chatgpt + backend-client)**

In `codex-rs/tui/Cargo.toml`, remove:
```toml
orbit-code-chatgpt = { workspace = true }
orbit-code-backend-client = { workspace = true }
```

In `codex-rs/tui/src/chatwidget.rs`:
- Replace `use orbit_code_backend_client::Client as BackendClient;` — remove import
- Replace `use orbit_code_chatgpt::connectors;` → `use orbit_code_core::connectors;`
- Stub the `fetch_rate_limits()` function (line ~9502) to return `Vec::new()` without calling `BackendClient`

In `codex-rs/tui/src/chatwidget/skills.rs`:
- Replace `use orbit_code_chatgpt::connectors::AppInfo;` → `use orbit_code_core::connectors::AppInfo;`

In `codex-rs/tui/src/bottom_pane/chat_composer.rs`:
- Replace `use orbit_code_chatgpt::connectors;` → `use orbit_code_core::connectors;`
- Replace `use orbit_code_chatgpt::connectors::AppInfo;` → `use orbit_code_core::connectors::AppInfo;`

In `codex-rs/tui/src/app_event.rs`:
- Replace `use orbit_code_chatgpt::connectors::AppInfo;` → `use orbit_code_core::connectors::AppInfo;`

In `codex-rs/tui/src/chatwidget/tests.rs`:
- Replace ALL `orbit_code_chatgpt::connectors::AppInfo` references with `orbit_code_core::connectors::AppInfo` (there are 20+ occurrences)

- [ ] **Step 7: Update tui_app_server crate (remove chatgpt)**

In `codex-rs/tui_app_server/Cargo.toml`, remove:
```toml
orbit-code-chatgpt = { workspace = true }
```

In `codex-rs/tui_app_server/src/chatwidget.rs`:
- Replace `use orbit_code_chatgpt::connectors;` → `use orbit_code_core::connectors;`

In `codex-rs/tui_app_server/src/chatwidget/skills.rs`:
- Replace `use orbit_code_chatgpt::connectors::AppInfo;` → `use orbit_code_core::connectors::AppInfo;`

In `codex-rs/tui_app_server/src/bottom_pane/chat_composer.rs`:
- Replace `use orbit_code_chatgpt::connectors;` → `use orbit_code_core::connectors;`
- Replace `use orbit_code_chatgpt::connectors::AppInfo;` → `use orbit_code_core::connectors::AppInfo;`

In `codex-rs/tui_app_server/src/app_event.rs`:
- Replace `use orbit_code_chatgpt::connectors::AppInfo;` → `use orbit_code_core::connectors::AppInfo;`

In `codex-rs/tui_app_server/src/chatwidget/tests.rs`:
- Replace ALL `orbit_code_chatgpt::connectors::AppInfo` references with `orbit_code_core::connectors::AppInfo` (there are 20+ occurrences)

- [ ] **Step 8: cargo check**

Run: `cd codex-rs && cargo check 2>&1 | head -50`
Expected: Errors only from cloud-requirements (Task 3). Fix any remaining compilation errors before proceeding.

- [ ] **Step 9: Commit**

```bash
git add codex-rs/
git commit -m "Remove connectors, chatgpt, backend-client, cloud-tasks, responses-api-proxy crates and all dependents"
```

---

### Task 3: Remove cloud-requirements (Most Dependents)

This crate provides `CloudRequirementsLoader` used by exec, app-server, tui, and tui_app_server. We replace calls with `CloudRequirementsLoader::default()` (no cloud requirements for Orbit Code — we'll build our own enterprise config system later if needed).

The `CloudRequirementsLoader` type is defined in `orbit-code-core::config_loader` and stays. We just stop providing an implementation — callers pass the default (which resolves to `None`).

**Files:**
- Delete: `codex-rs/cloud-requirements/`
- Modify: `codex-rs/Cargo.toml` — remove member + dep
- Modify: `codex-rs/exec/Cargo.toml` + `src/lib.rs`
- Modify: `codex-rs/app-server/Cargo.toml` + `src/lib.rs` + `src/orbit_code_message_processor.rs`
- Modify: `codex-rs/tui/Cargo.toml` + `src/lib.rs`
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
- Replace `cloud_requirements_loader(cloud_auth_manager, chatgpt_base_url, orbit_code_home.clone())` with `CloudRequirementsLoader::default()`
- Replace `run_cloud_requirements = cloud_requirements.clone()` — just use the default
- Add `use orbit_code_core::config_loader::CloudRequirementsLoader;` if not already imported
- All `.cloud_requirements(cloud_requirements)` builder calls become `.cloud_requirements(CloudRequirementsLoader::default())` or simply remove them if the builder defaults to `None`

- [ ] **Step 4: Update app-server crate**

In `codex-rs/app-server/Cargo.toml`, remove:
```toml
orbit-code-cloud-requirements = { workspace = true }
```

In `codex-rs/app-server/src/lib.rs`:
- Remove `use orbit_code_cloud_requirements::cloud_requirements_loader;`
- Replace the `cloud_requirements_loader(...)` call (~line 428) with `CloudRequirementsLoader::default()`
- Add `use orbit_code_core::config_loader::CloudRequirementsLoader;` if not already imported
- Update all `.cloud_requirements(cloud_requirements)` builder calls

In `codex-rs/app-server/src/orbit_code_message_processor.rs`:
- Remove `use orbit_code_cloud_requirements::cloud_requirements_loader;`
- Replace `cloud_requirements_loader(...)` call with `CloudRequirementsLoader::default()`

- [ ] **Step 5: Update tui crate**

In `codex-rs/tui/Cargo.toml`, remove:
```toml
orbit-code-cloud-requirements = { workspace = true }
```

In `codex-rs/tui/src/lib.rs` — this has **extensive** usage (15+ references):
- Remove `use orbit_code_cloud_requirements::cloud_requirements_loader;` (line 13)
- Add `use orbit_code_core::config_loader::CloudRequirementsLoader;` if not already imported
- Replace `cloud_requirements_loader(cloud_auth_manager, chatgpt_base_url, orbit_code_home.to_path_buf())` (line ~367) with `CloudRequirementsLoader::default()`
- Replace `cloud_requirements = cloud_requirements_loader(auth_manager.clone(), ...)` (line ~669) with `cloud_requirements = CloudRequirementsLoader::default()`
- Update the `mut cloud_requirements: CloudRequirementsLoader` parameter in `run_ratatui_app()` (line ~582) — keep the type, callers now pass default
- Update ALL `.cloud_requirements(cloud_requirements)` and `.cloud_requirements(cloud_requirements.clone())` builder calls (lines ~428, ~682, ~943, ~1183, ~1192, ~1199)
- Update `load_config_or_exit()` and `load_config_or_exit_with_fallback_cwd()` function signatures — they accept `cloud_requirements: CloudRequirementsLoader`

- [ ] **Step 6: Update tui_app_server crate**

In `codex-rs/tui_app_server/Cargo.toml`, remove:
```toml
orbit-code-cloud-requirements = { workspace = true }
```

In `codex-rs/tui_app_server/src/lib.rs` — this also has extensive usage:
- Remove `use orbit_code_cloud_requirements::cloud_requirements_loader_for_storage;` (line ~27)
- Add `use orbit_code_core::config_loader::CloudRequirementsLoader;` if not already imported
- Replace `cloud_requirements_loader_for_storage(...)` call (line ~689) with `CloudRequirementsLoader::default()`
- Replace the second `cloud_requirements_loader_for_storage(...)` call (line ~1030) with `CloudRequirementsLoader::default()`
- Update the `mut cloud_requirements: CloudRequirementsLoader` parameter and all usage sites — same pattern as tui
- Update ALL `.cloud_requirements(cloud_requirements)` and `.cloud_requirements(cloud_requirements.clone())` builder calls
- Update `load_config_or_exit()` and `load_config_or_exit_with_fallback_cwd()` function signatures

- [ ] **Step 7: Check if CloudRequirementsLoader type is defined in core**

The `CloudRequirementsLoader` type is defined in `orbit-code-core::config_loader`. It stays (it's a type that the `ConfigBuilder` accepts). We just stop providing a real loader — callers pass `CloudRequirementsLoader::default()` instead of a real one.

Run: `grep -rn 'CloudRequirementsLoader' codex-rs/core/src/ | head -10`

If it's a type in core that has a `Default` impl, use `CloudRequirementsLoader::default()`. If it accepts `Option`, pass `None`. Check the actual API.

Also check `codex-rs/core/src/connectors_tests.rs` for `CloudRequirementsLoader::new(async move { ... })` test usage and update if needed.

- [ ] **Step 8: cargo check**

Run: `cd codex-rs && cargo check`
Expected: Clean compilation

- [ ] **Step 9: Commit**

```bash
git add codex-rs/
git commit -m "Remove cloud-requirements crate — pass default loader for cloud config"
```

---

### Task 4: Clean Up npm Package for responses-api-proxy

**Files:**
- Delete: `codex-rs/responses-api-proxy/npm/` (if not already deleted with parent in Task 2)
- Modify: `pnpm-workspace.yaml` — remove responses-api-proxy/npm entry

- [ ] **Step 1: Remove from pnpm workspace**

In `pnpm-workspace.yaml`, remove:
```yaml
  - codex-rs/responses-api-proxy/npm
```

- [ ] **Step 2: Commit**

```bash
git add pnpm-workspace.yaml
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

Run all workspace tests, not just core and cli — the TUI crates have extensive connector tests that need verification:

```bash
cd codex-rs && cargo test -p orbit-code-core 2>&1 | tail -20
cd codex-rs && cargo test -p orbit-code-cli 2>&1 | tail -20
cd codex-rs && cargo test -p orbit-code-app-server 2>&1 | tail -20
cd codex-rs && cargo test -p orbit-code-exec 2>&1 | tail -20
cd codex-rs && cargo test -p orbit-code-tui 2>&1 | tail -20
cd codex-rs && cargo test -p orbit-code-tui-app-server 2>&1 | tail -20
```
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

- [ ] **Step 6: Verify no stale Cargo.toml dependency entries**

```bash
grep -rn 'orbit-code-chatgpt\|orbit-code-backend-client\|orbit-code-cloud-tasks\|orbit-code-cloud-requirements\|orbit-code-connectors\|orbit-code-responses-api-proxy\|orbit-code-backend-openapi' \
  codex-rs/ --include='Cargo.toml' | grep -v target/ | head -10
```
Expected: No output

- [ ] **Step 7: Commit and push**

```bash
git add codex-rs/
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

## Follow-up Items (Stage 3 or later)

- **`chatgpt_base_url` config field** — Now dead after removing all chatgpt/cloud-requirements callers. Can be removed from config types in a future cleanup.
- **`local_chatgpt_auth.rs` in tui_app_server** — Module name references "chatgpt" but only depends on `core::auth` and `app-server-protocol`. No functional dependency on removed crates, but consider renaming.
- **Rate limit display** — Both TUI and app-server rate limit features are stubbed to return empty. Implement against Orbit backend when available.
- **Bazel BUILD.bazel files** — `backend-client/BUILD.bazel` and other removed crates may have Bazel configs. If Bazel builds are used in CI, verify they still work.
