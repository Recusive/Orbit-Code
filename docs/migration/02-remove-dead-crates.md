# Stage 2: Remove Dead Crates

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove 8 OpenAI-only crates that will never be used in Orbit Code. Migrate any reusable logic to its long-term owner first, then delete crates and clean all references so the project compiles and tests pass.

**Architecture:** Behavior-preserving migration first, then bottom-up deletion. Move generic connector helpers from `chatgpt::connectors` into `core::connectors` before deleting `chatgpt`. Then delete leaf crates, mid-level crates, and finally `cloud-requirements`.

**Tech Stack:** Rust (Cargo workspace), file deletion, targeted source edits, npm/pnpm workspace, Bazel lock maintenance.

**Depends on:** Stage 1 (rename) must be complete. Crate names below use post-rename names (`orbit-code-*`).

---

## Public Surface Changes

This plan intentionally removes the following user-facing capabilities:

| Surface | Current Behavior | After Stage 2 | Rationale |
|---------|-----------------|---------------|-----------|
| `orbit-code apply` CLI | Fetches and applies diffs from ChatGPT cloud tasks | **Removed** | ChatGPT-specific; Orbit will have its own task system |
| `orbit-code cloud` CLI | Browses ChatGPT cloud tasks | **Removed** | ChatGPT-specific |
| `orbit-code responses-api-proxy` | Runs OpenAI responses API proxy | **Removed** | OpenAI-specific internal tool |
| ChatGPT directory app listing | `app/list` returns ChatGPT directory + MCP apps | `app/list` returns **MCP-discovered apps only** | Directory API is ChatGPT-specific; MCP discovery is the Orbit path |
| Plugin metadata from directory | `plugin/install` shows ChatGPT directory metadata | Plugin metadata from **MCP tools only** | Same as above |
| Rate limits from OpenAI backend | TUI + app-server show OpenAI rate limit data | Rate limit display returns **empty** | OpenAI backend API; stub until Orbit backend supports this |
| Managed cloud requirements | Business/Enterprise requirements enforced via ChatGPT backend | Requirements loader returns **None** | ChatGPT-specific managed config; Orbit will build its own enterprise config if needed |
| `configRequirements/read` | Returns ChatGPT-managed requirements | Returns **None** (meaning "no managed requirements configured") | Same as above |
| `tool_suggest` installable connectors | Suggests installable connectors from ChatGPT directory | Suggests **only MCP-accessible connectors** — directory-only installable connectors no longer appear in tool suggestions | Directory discoverables came from `list_tool_suggest_discoverable_tools_with_auth` which joined directory + accessible lists; without a directory source, only MCP-discovered tools remain |
| Synthetic plugin app summaries | Plugin apps enriched with directory metadata (description, logo, install URL) | Plugin apps have **ID-only synthetic entries** — connector ID used as name, no description/logo/install URL | `merge_plugin_apps` creates placeholder entries from plugin app IDs; Orbit clients may want copy changes if they display plugin metadata to users |

**What keeps working:**
- MCP-based app/connector discovery (`list_accessible_connectors_from_mcp_tools` — already in core)
- All connector UI in TUI (app picker, `$` mentions, skills) — works with MCP-discovered apps
- App enable/disable config — still in core
- Plugin install/auth flow — works; `connectors_for_plugin_apps` creates synthetic entries from plugin app IDs (without enriched directory metadata like description/logo/install URL)
- `CloudRequirementsLoader` type — stays in `core::config_loader`, callers pass `default()`
- `/apps` merge logic — directory stubs return `None`/`Err` (not empty `Ok`), so `merge_loaded_apps` sets `all_connectors_loaded = false` and keeps all MCP accessible connectors unfiltered
- `configRequirements/read` — returns `None`, meaning "no managed requirements configured" (not "feature removed")

---

## Crates to Remove

| # | Crate (post-rename) | Path | Live Dependents (outside dead set) | Removal Complexity |
|---|---------------------|------|-----------------------------------|--------------------|
| 1 | `orbit-code-backend-openapi-models` | `codex-rs/codex-backend-openapi-models/` | none | EASY — true leaf |
| 2 | `orbit-code-cloud-tasks-client` | `codex-rs/cloud-tasks-client/` | none | EASY — true leaf |
| 3 | `orbit-code-connectors` | `codex-rs/connectors/` | **core** | MEDIUM — core uses types + `list_all_connectors_with_options()` |
| 4 | `orbit-code-chatgpt` | `codex-rs/chatgpt/` | cli, app-server, **tui, tui_app_server** | HARD — connector functions + AppInfo type across 8+ source files |
| 5 | `orbit-code-backend-client` | `codex-rs/backend-client/` | **tui, app-server** | MEDIUM — rate limit fetching in chatwidget + message processor |
| 6 | `orbit-code-cloud-tasks` | `codex-rs/cloud-tasks/` | cli | MEDIUM — remove CLI subcommand |
| 7 | `orbit-code-responses-api-proxy` | `codex-rs/responses-api-proxy/` | cli | MEDIUM — remove CLI subcommand + npm package + build scripts |
| 8 | `orbit-code-cloud-requirements` | `codex-rs/cloud-requirements/` | exec, app-server, tui, tui_app_server | HARDEST — 4 crates need `CloudRequirementsLoader::default()` replacement |

## Dependency Graph (removal order)

```
Pre-work (behavior-preserving migration):
  Move merge_connectors_with_accessible() and connectors_for_plugin_apps()
  from chatgpt::connectors → core::connectors

Delete first (true leaves — no live dependents outside dead set):
  backend-openapi-models ──→ (only used by backend-client, which is also dead)
  cloud-tasks-client ──→ (only used by cloud-tasks, which is also dead)

Delete second (mid-level — live dependents need source edits):
  connectors ──→ (core [LIVE] — needs inline stubs for types + stub for directory fetch)
  chatgpt ──→ (cli, app-server, tui, tui_app_server — switch to core::connectors)
  backend-client ──→ (tui [LIVE], app-server [LIVE] — stub rate limit functions)
  cloud-tasks ──→ (cli — remove subcommand)
  responses-api-proxy ──→ (cli — remove subcommand; npm scripts — remove packaging)

Delete last (most dependents):
  cloud-requirements ──→ (exec, app-server, tui, tui_app_server — use default loader)
```

---

## Files Modified

**Workspace root:**
- `codex-rs/Cargo.toml` — remove 8 members + workspace.dependencies entries

**Core crate (pre-migration + connectors dep removal):**
- `codex-rs/core/Cargo.toml` — remove `orbit-code-connectors` dependency
- `codex-rs/core/src/connectors.rs` — add `merge_connectors_with_accessible()` and `connectors_for_plugin_apps()`; inline `AllConnectorsCacheKey`, `DirectoryListResponse`, `CONNECTORS_CACHE_TTL`; stub `list_all_connectors_with_options()` call site
- `codex-rs/core/src/connectors_tests.rs` — update `CloudRequirementsLoader` test usage if needed

**CLI crate (remove subcommands):**
- `codex-rs/cli/Cargo.toml` — remove 3 dependencies
- `codex-rs/cli/src/main.rs` — remove `apply`, `cloud`, `responses-api-proxy` subcommands and imports

**App-server (remove chatgpt, backend-client, cloud-requirements):**
- `codex-rs/app-server/Cargo.toml` — remove 4 dependencies
- `codex-rs/app-server/src/lib.rs` — remove cloud_requirements_loader usage
- `codex-rs/app-server/src/orbit_code_message_processor.rs` — switch `connectors::` calls to `core::connectors`; remove `BackendClient`; stub rate limits; remove `cloud_requirements_loader`
- `codex-rs/app-server/src/orbit_code_message_processor/apps_list_helpers.rs` — update `connectors::` import if needed
- `codex-rs/app-server/src/orbit_code_message_processor/plugin_app_helpers.rs` — update `connectors::` calls to use core versions

**Exec (remove cloud-requirements):**
- `codex-rs/exec/Cargo.toml` — remove 1 dependency
- `codex-rs/exec/src/lib.rs` — remove cloud_requirements_loader import + usage

**TUI (remove chatgpt, backend-client, cloud-requirements):**
- `codex-rs/tui/Cargo.toml` — remove 3 dependencies
- `codex-rs/tui/src/lib.rs` — remove `cloud_requirements_loader` import + all 15+ usages
- `codex-rs/tui/src/chatwidget.rs` — replace `BackendClient` and `chatgpt::connectors` imports
- `codex-rs/tui/src/chatwidget/skills.rs` — replace `chatgpt::connectors::AppInfo` import
- `codex-rs/tui/src/bottom_pane/chat_composer.rs` — replace `chatgpt::connectors` imports
- `codex-rs/tui/src/app_event.rs` — replace `chatgpt::connectors::AppInfo` import
- `codex-rs/tui/src/chatwidget/tests.rs` — replace all `chatgpt::connectors::AppInfo` usages (20+)

**TUI App Server (remove chatgpt, cloud-requirements):**
- `codex-rs/tui_app_server/Cargo.toml` — remove 2 dependencies
- `codex-rs/tui_app_server/src/lib.rs` — remove `cloud_requirements_loader_for_storage` import + all usages
- `codex-rs/tui_app_server/src/onboarding/auth.rs` — remove `cloud_requirements_loader_for_storage` import + usage in `#[cfg(test)]` module (3 tests affected)
- `codex-rs/tui_app_server/src/chatwidget.rs` — replace `chatgpt::connectors` import
- `codex-rs/tui_app_server/src/chatwidget/skills.rs` — replace import
- `codex-rs/tui_app_server/src/bottom_pane/chat_composer.rs` — replace imports
- `codex-rs/tui_app_server/src/app_event.rs` — replace import
- `codex-rs/tui_app_server/src/chatwidget/tests.rs` — replace all usages (20+)

**Documentation:**
- `docs/codex/config.md` — remove/update ChatGPT connector, apps directory, and managed requirements references
- `codex-rs/app-server/README.md` — update `app/list`, `configRequirements/read`, and rate-limit endpoint docs

**Packaging + lockfiles:**
- `pnpm-workspace.yaml` — remove `codex-rs/responses-api-proxy/npm` entry
- `pnpm-lock.yaml` — regenerate via `pnpm install --lockfile-only`
- `codex-cli/scripts/build_npm_package.py` — remove `codex-responses-api-proxy` package handling
- `codex-cli/scripts/install_native_deps.py` — remove `codex-responses-api-proxy` BinaryComponent
- `codex-cli/scripts/README.md` — remove proxy example

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

### Task 1: Migrate Connector Logic to Core (Behavior-Preserving)

Before deleting crates, move the two pure-logic connector functions from `chatgpt::connectors` into `core::connectors`. This ensures the import targets exist before callers switch to them.

**Why:** `chatgpt::connectors` contains wrapper functions that callers (`app-server`, `tui`, `tui_app_server`) import. Most are re-exports from `core::connectors`, but two pure-logic functions live only in `chatgpt::connectors`:
- `merge_connectors_with_accessible(all, accessible, all_loaded)` — merges directory + MCP connectors
- `connectors_for_plugin_apps(connectors, plugin_apps)` — filters to plugin-requested apps

These use `merge_connectors`, `merge_plugin_apps`, and `filter_disallowed_connectors` which are **already in core**.

**Files:**
- Modify: `codex-rs/core/src/connectors.rs` — add two functions

- [ ] **Step 1: Add `merge_connectors_with_accessible` to core**

In `codex-rs/core/src/connectors.rs`, add (near the existing `merge_connectors` function):

```rust
pub fn merge_connectors_with_accessible(
    connectors: Vec<AppInfo>,
    accessible_connectors: Vec<AppInfo>,
    all_connectors_loaded: bool,
) -> Vec<AppInfo> {
    let accessible_connectors = if all_connectors_loaded {
        let connector_ids: HashSet<&str> = connectors
            .iter()
            .map(|connector| connector.id.as_str())
            .collect();
        accessible_connectors
            .into_iter()
            .filter(|connector| connector_ids.contains(connector.id.as_str()))
            .collect()
    } else {
        accessible_connectors
    };
    let merged = merge_connectors(connectors, accessible_connectors);
    filter_disallowed_connectors(merged)
}
```

- [ ] **Step 2: Add `connectors_for_plugin_apps` to core**

In `codex-rs/core/src/connectors.rs`, add:

```rust
pub fn connectors_for_plugin_apps(
    connectors: Vec<AppInfo>,
    plugin_apps: &[AppConnectorId],
) -> Vec<AppInfo> {
    let plugin_app_ids = plugin_apps
        .iter()
        .map(|connector_id| connector_id.0.as_str())
        .collect::<HashSet<_>>();

    filter_disallowed_connectors(merge_plugin_apps(connectors, plugin_apps.to_vec()))
        .into_iter()
        .filter(|connector| plugin_app_ids.contains(connector.id.as_str()))
        .collect()
}
```

**Note on plugin behavior without directory:** When `list_all_connectors_with_options` returns `Err`, `plugin_app_helpers.rs` falls back to `list_cached_all_connectors` which returns `None`, then `unwrap_or_default()` gives `vec![]`. Then `connectors_for_plugin_apps(vec![], plugin_apps)` calls `merge_plugin_apps` which creates **synthetic AppInfo entries** for each plugin app (using the connector ID as name). Plugins still get basic entries — just without enriched directory metadata (description, logo, install URL). This is acceptable for Orbit.

- [ ] **Step 3: Add stubs for directory connector functions in core**

The `chatgpt::connectors` module provides directory-listing functions that fetch from the ChatGPT backend. Orbit has no ChatGPT directory, so these become stubs. **The stub signatures must signal "directory unavailable," not "directory is empty,"** because the merge logic in `merge_connectors_with_accessible()` treats `all_connectors_loaded = true` + empty list as "filter out all accessible connectors not in directory." Returning `Err` / `None` keeps `all_connectors_loaded = false`, which preserves MCP-discovered apps unfiltered.

```rust
/// Stub: ChatGPT connector directory not available in Orbit.
/// Returns Err so callers treat this as "directory unavailable" rather than
/// "directory returned zero connectors" — the distinction matters because
/// merge_connectors_with_accessible() filters accessible connectors against
/// the directory list when all_connectors_loaded is true.
pub async fn list_all_connectors_with_options(
    _config: &Config,
    _force_refetch: bool,
) -> anyhow::Result<Vec<AppInfo>> {
    anyhow::bail!("ChatGPT connector directory not available in Orbit")
}

/// Stub: no cached ChatGPT directory data.
/// Returns None (not Some(vec![])) so merge_loaded_apps() sets
/// all_connectors_loaded = false, preserving MCP accessible connectors.
pub async fn list_cached_all_connectors(_config: &Config) -> Option<Vec<AppInfo>> {
    None
}

/// Stub: returns only MCP-discovered connectors (no ChatGPT directory merge).
pub async fn list_connectors(config: &Config) -> anyhow::Result<Vec<AppInfo>> {
    let accessible = list_accessible_connectors_from_mcp_tools(config).await?;
    Ok(with_app_enabled_state(accessible, config))
}
```

**Why Err/None instead of empty Ok/Some:**

The app-server merge path works like this:
1. `list_cached_all_connectors()` → if `Some(vec![])`, sets `all_connectors = Some([])`
2. `merge_loaded_apps(Some(&[]), Some(&[mcp_apps...]))` → `all_connectors_loaded = true`
3. `merge_connectors_with_accessible([], [mcp_apps], true)` → **filters MCP apps against empty directory → returns empty!**

With `None` instead:
1. `list_cached_all_connectors()` → `None`, sets `all_connectors = None`
2. `merge_loaded_apps(None, Some(&[mcp_apps...]))` → `all_connectors_loaded = false`
3. `merge_connectors_with_accessible([], [mcp_apps], false)` → **keeps all MCP apps unfiltered**

Similarly, `list_all_connectors_with_options` returning `Err` means the async spawned task error-handling path leaves `all_connectors` as `None`.

- [ ] **Step 4: cargo check**

Run: `cd codex-rs && cargo check -p orbit-code-core 2>&1 | head -30`
Expected: Clean. The new functions should compile since they use types already in scope.

- [ ] **Step 5: Commit**

```bash
git add codex-rs/core/src/connectors.rs
git commit -m "Migrate connector helpers from chatgpt to core — prep for crate deletion"
```

---

### Task 2: Delete Leaf Crates

Only `backend-openapi-models` and `cloud-tasks-client` are true leaves.

- [ ] **Step 1: Delete directories**

```bash
rm -rf codex-rs/codex-backend-openapi-models/
rm -rf codex-rs/cloud-tasks-client/
```

- [ ] **Step 2: Remove from workspace Cargo.toml**

Remove from `[workspace] members` in `codex-rs/Cargo.toml`:
- `"codex-backend-openapi-models"`
- `"cloud-tasks-client"`

Remove from `[workspace.dependencies]` if present (check first — they may not have entries).

- [ ] **Step 3: cargo check + commit**

```bash
cd codex-rs && cargo check 2>&1 | head -30
git add codex-rs/Cargo.toml codex-rs/Cargo.lock
git commit -m "Remove leaf crates: backend-openapi-models, cloud-tasks-client"
```

---

### Task 3: Delete connectors, chatgpt, backend-client, cloud-tasks, responses-api-proxy

These crates have live dependents. After Task 1's migration, callers switch from `chatgpt::connectors` to `core::connectors`.

- [ ] **Step 1: Delete directories**

```bash
rm -rf codex-rs/connectors/
rm -rf codex-rs/chatgpt/
rm -rf codex-rs/backend-client/
rm -rf codex-rs/cloud-tasks/
rm -rf codex-rs/responses-api-proxy/
```

- [ ] **Step 2: Update workspace Cargo.toml**

Remove from members: `"connectors"`, `"backend-client"`, `"cloud-tasks"`, `"responses-api-proxy"`
(Check if `"chatgpt"` is in members — remove if present.)

Remove from workspace.dependencies:
- `orbit-code-connectors = { path = "connectors" }`
- `orbit-code-chatgpt = { path = "chatgpt" }`
- `orbit-code-backend-client = { path = "backend-client" }`
- `orbit-code-responses-api-proxy = { path = "responses-api-proxy" }`

(Note: `orbit-code-cloud-tasks` is NOT in workspace.dependencies — the CLI uses a direct path dep.)

- [ ] **Step 3: Update core (remove connectors crate dependency)**

In `codex-rs/core/Cargo.toml`, remove:
```toml
orbit-code-connectors = { workspace = true }
```

In `codex-rs/core/src/connectors.rs`:
- Remove `use orbit_code_connectors::AllConnectorsCacheKey;`
- Remove `use orbit_code_connectors::DirectoryListResponse;`
- Remove `pub use orbit_code_connectors::CONNECTORS_CACHE_TTL;`
- Add inline constant: `pub const CONNECTORS_CACHE_TTL: Duration = Duration::from_secs(3600);`
- The `orbit_code_connectors::list_all_connectors_with_options()` call site (~line 435) was replaced by the stub in Task 1 Step 3. If there's still a call site, remove it — the stub function now returns empty.
- Inline `AllConnectorsCacheKey` and `DirectoryListResponse` type definitions only if they're still needed by remaining code. If Task 1's stubs eliminated all usage, these can be deleted entirely.

Check `codex-rs/core/src/connectors_tests.rs` for any `orbit_code_connectors` imports and update.

- [ ] **Step 4: Update CLI**

In `codex-rs/cli/Cargo.toml`, remove:
```toml
orbit-code-chatgpt = { workspace = true }
orbit-code-cloud-tasks = { path = "../cloud-tasks" }
orbit-code-responses-api-proxy = { workspace = true }
```

In `codex-rs/cli/src/main.rs`:
- Remove imports: `ApplyCommand`, `run_apply_command` from `orbit_code_chatgpt`
- Remove imports: `CloudTasksCli` from `orbit_code_cloud_tasks`
- Remove imports: `ResponsesApiProxyArgs` from `orbit_code_responses_api_proxy`
- Remove `Apply`, `Cloud`, `ResponsesApiProxy` variants from the `Subcommand` enum
- Remove their match arms in `cli_main()`

- [ ] **Step 5: Update app-server (remove chatgpt + backend-client)**

In `codex-rs/app-server/Cargo.toml`, remove:
```toml
orbit-code-chatgpt = { workspace = true }
orbit-code-backend-client = { workspace = true }
```

In `codex-rs/app-server/src/orbit_code_message_processor.rs`:
- Remove `use orbit_code_backend_client::Client as BackendClient;`
- Replace `use orbit_code_chatgpt::connectors;` → `use orbit_code_core::connectors;`
- Stub `BackendClient::from_auth()` call sites (rate limit fetching) to return empty results
- Verify that `connectors::list_all_connectors_with_options`, `connectors::list_cached_all_connectors`, `connectors::merge_connectors_with_accessible`, `connectors::connectors_for_plugin_apps`, `connectors::list_connectors` all resolve from `core::connectors` (they should after Task 1)

In `codex-rs/app-server/src/orbit_code_message_processor/apps_list_helpers.rs`:
- Update `connectors::merge_connectors_with_accessible` to resolve from `core::connectors` (via the parent module's updated import)

In `codex-rs/app-server/src/orbit_code_message_processor/plugin_app_helpers.rs`:
- Update `connectors::list_all_connectors_with_options`, `connectors::list_cached_all_connectors`, `connectors::connectors_for_plugin_apps` to resolve from `core::connectors`

- [ ] **Step 6: Update TUI (remove chatgpt + backend-client)**

In `codex-rs/tui/Cargo.toml`, remove:
```toml
orbit-code-chatgpt = { workspace = true }
orbit-code-backend-client = { workspace = true }
```

Source file changes — replace all `orbit_code_chatgpt::connectors` with `orbit_code_core::connectors`:

| File | Change |
|------|--------|
| `src/chatwidget.rs` | Remove `use orbit_code_backend_client::Client as BackendClient;`; replace `use orbit_code_chatgpt::connectors;` → `use orbit_code_core::connectors;`; stub `fetch_rate_limits()` to return `Vec::new()` |
| `src/chatwidget/skills.rs` | Replace `use orbit_code_chatgpt::connectors::AppInfo;` → `use orbit_code_core::connectors::AppInfo;` |
| `src/bottom_pane/chat_composer.rs` | Replace both `orbit_code_chatgpt::connectors` imports → `orbit_code_core::connectors` |
| `src/app_event.rs` | Replace `use orbit_code_chatgpt::connectors::AppInfo;` → `use orbit_code_core::connectors::AppInfo;` |
| `src/chatwidget/tests.rs` | Replace ALL `orbit_code_chatgpt::connectors::AppInfo` (20+) → `orbit_code_core::connectors::AppInfo` |

- [ ] **Step 7: Update tui_app_server (remove chatgpt)**

In `codex-rs/tui_app_server/Cargo.toml`, remove:
```toml
orbit-code-chatgpt = { workspace = true }
```

Same import replacement pattern as TUI:

| File | Change |
|------|--------|
| `src/chatwidget.rs` | Replace `use orbit_code_chatgpt::connectors;` → `use orbit_code_core::connectors;` |
| `src/chatwidget/skills.rs` | Replace `AppInfo` import |
| `src/bottom_pane/chat_composer.rs` | Replace both `connectors` imports |
| `src/app_event.rs` | Replace `AppInfo` import |
| `src/chatwidget/tests.rs` | Replace ALL `AppInfo` references (20+) |

- [ ] **Step 8: cargo check**

Run: `cd codex-rs && cargo check 2>&1 | head -50`
Expected: Errors only from cloud-requirements (Task 4). Fix any remaining errors before proceeding.

- [ ] **Step 9: Commit**

```bash
git add codex-rs/
git commit -m "Remove connectors, chatgpt, backend-client, cloud-tasks, responses-api-proxy — migrate callers to core"
```

---

### Task 4: Delete cloud-requirements

This crate provides `CloudRequirementsLoader` implementations used by exec, app-server, tui, and tui_app_server. Orbit does not use ChatGPT Business/Enterprise managed requirements. Replace with `CloudRequirementsLoader::default()` which resolves to `Ok(None)`.

The `CloudRequirementsLoader` **type** is defined in `orbit-code-core::config_loader` and stays.

- [ ] **Step 1: Delete directory**

```bash
rm -rf codex-rs/cloud-requirements/
```

- [ ] **Step 2: Update workspace Cargo.toml**

Remove from members: `"cloud-requirements"`
Remove from deps: `orbit-code-cloud-requirements = { path = "cloud-requirements" }`

- [ ] **Step 3: Update exec**

In `codex-rs/exec/Cargo.toml`, remove:
```toml
orbit-code-cloud-requirements = { workspace = true }
```

In `codex-rs/exec/src/lib.rs`:
- Remove `use orbit_code_cloud_requirements::cloud_requirements_loader;`
- Add `use orbit_code_core::config_loader::CloudRequirementsLoader;` if not already imported
- Replace `cloud_requirements_loader(cloud_auth_manager, chatgpt_base_url, orbit_code_home.clone())` → `CloudRequirementsLoader::default()`
- Simplify: remove `run_cloud_requirements` variable if it just holds the default
- Update all `.cloud_requirements(...)` builder calls to use the default

- [ ] **Step 4: Update app-server**

In `codex-rs/app-server/Cargo.toml`, remove:
```toml
orbit-code-cloud-requirements = { workspace = true }
```

In `codex-rs/app-server/src/lib.rs`:
- Remove `use orbit_code_cloud_requirements::cloud_requirements_loader;`
- Replace `cloud_requirements_loader(...)` → `CloudRequirementsLoader::default()`
- Update all `.cloud_requirements(...)` builder calls

In `codex-rs/app-server/src/orbit_code_message_processor.rs`:
- Remove `use orbit_code_cloud_requirements::cloud_requirements_loader;`
- Replace `cloud_requirements_loader(...)` → `CloudRequirementsLoader::default()`
- The auth-refresh path that rebuilds `cloud_requirements_loader` after login/logout → just use `CloudRequirementsLoader::default()` again

- [ ] **Step 5: Update tui**

In `codex-rs/tui/Cargo.toml`, remove:
```toml
orbit-code-cloud-requirements = { workspace = true }
```

In `codex-rs/tui/src/lib.rs` — **extensive** usage (15+ references):
- Remove `use orbit_code_cloud_requirements::cloud_requirements_loader;` (line 13)
- Add `use orbit_code_core::config_loader::CloudRequirementsLoader;`
- Replace `cloud_requirements_loader(cloud_auth_manager, chatgpt_base_url, ...)` (line ~367) → `CloudRequirementsLoader::default()`
- Replace `cloud_requirements = cloud_requirements_loader(auth_manager.clone(), ...)` (line ~669) → `cloud_requirements = CloudRequirementsLoader::default()`
- Keep the `mut cloud_requirements: CloudRequirementsLoader` parameter type in `run_ratatui_app()`
- Update ALL `.cloud_requirements(cloud_requirements.clone())` builder calls
- Update `load_config_or_exit()` and `load_config_or_exit_with_fallback_cwd()` signatures and callers

- [ ] **Step 6: Update tui_app_server**

In `codex-rs/tui_app_server/Cargo.toml`, remove:
```toml
orbit-code-cloud-requirements = { workspace = true }
```

In `codex-rs/tui_app_server/src/lib.rs` — same extensive pattern:
- Remove `use orbit_code_cloud_requirements::cloud_requirements_loader_for_storage;`
- Add `use orbit_code_core::config_loader::CloudRequirementsLoader;`
- Replace both `cloud_requirements_loader_for_storage(...)` calls → `CloudRequirementsLoader::default()`
- Update all `.cloud_requirements(...)` builder calls
- Update `load_config_or_exit()` and `load_config_or_exit_with_fallback_cwd()` signatures

In `codex-rs/tui_app_server/src/onboarding/auth.rs` — has a `#[cfg(test)]` module (line ~883) that imports `cloud_requirements_loader_for_storage`:
- Remove `use orbit_code_cloud_requirements::cloud_requirements_loader_for_storage;` (line ~891)
- Replace the `cloud_requirements_loader_for_storage(...)` call in `widget_forced_chatgpt()` (lines ~912-917) with `CloudRequirementsLoader::default()`
- This affects 3 test functions: `api_key_flow_disabled_when_chatgpt_forced`, `saving_api_key_is_blocked_when_chatgpt_forced`, `existing_chatgpt_auth_tokens_login_counts_as_signed_in`

- [ ] **Step 7: cargo check + commit**

```bash
cd codex-rs && cargo check
git add codex-rs/
git commit -m "Remove cloud-requirements crate — Orbit uses default loader (no managed cloud config)"
```

---

### Task 5: Packaging, Lockfiles, and Build Script Cleanup

- [ ] **Step 1: Clean pnpm workspace**

In `pnpm-workspace.yaml`, remove:
```yaml
  - codex-rs/responses-api-proxy/npm
```

- [ ] **Step 2: Clean build scripts**

In `codex-cli/scripts/build_npm_package.py`:
- Remove the `RESPONSES_API_PROXY_NPM_ROOT` constant (line ~15)
- Remove `"codex-responses-api-proxy"` from `PACKAGE_NATIVE_COMPONENTS` dict (line ~78)
- Remove `"codex-responses-api-proxy"` from `COMPONENT_DEST_DIR` dict (line ~91)
- Remove the `if package == "codex-responses-api-proxy"` conditional blocks (lines ~191-196 and ~282-292)

In `codex-cli/scripts/install_native_deps.py`:
- Remove the `"codex-responses-api-proxy": BinaryComponent(...)` entry from `BINARY_COMPONENTS` (lines ~52-56)

In `codex-cli/scripts/README.md`:
- Remove the `--package codex-responses-api-proxy` example line

- [ ] **Step 3: Regenerate lockfiles**

```bash
# Cargo lockfile
cd codex-rs && cargo generate-lockfile

# pnpm lockfile
pnpm install --lockfile-only

# Bazel lockfile
just bazel-lock-update
```

- [ ] **Step 4: Verify Bazel lock**

```bash
just bazel-lock-check
```

- [ ] **Step 5: Commit**

```bash
git add codex-rs/Cargo.lock pnpm-workspace.yaml pnpm-lock.yaml MODULE.bazel.lock codex-cli/scripts/
git commit -m "Clean up packaging, build scripts, and lockfiles after crate removal"
```

---

### Task 6: Update Documentation

Public docs must ship with the code change so the documented behavior matches the actual behavior.

- [ ] **Step 1: Update `docs/codex/config.md`**

Remove or update references to ChatGPT connectors, apps directory, and managed cloud requirements. The app/connector story is now MCP-only.

- [ ] **Step 2: Update `codex-rs/app-server/README.md`**

Update documentation for:
- `app/list` — now returns MCP-discovered apps only, no ChatGPT directory enrichment
- `configRequirements/read` — now returns `null` (no managed requirements configured)
- Rate-limit endpoints — now return empty results

- [ ] **Step 3: Commit**

```bash
git add docs/codex/config.md codex-rs/app-server/README.md
git commit -m "Update docs for post-crate-removal behavior: MCP-only apps, no managed requirements"
```

---

### Task 7: Add Targeted Regression Tests and Verify

- [ ] **Step 1: Add targeted regression tests for new semantics**

Add tests that assert the documented post-removal behavior rather than relying only on broad crate suites:

**In `codex-rs/app-server/tests/` (or inline in `orbit_code_message_processor.rs`):**
- Add a test asserting that `app/list` returns MCP-accessible apps when no directory source exists (i.e., `list_all_connectors_with_options` returns `Err` and `list_cached_all_connectors` returns `None`, but MCP tools provide accessible connectors → the merged result contains those MCP apps)
- Add a test asserting that `plugin/install` accepts synthetic plugin app summaries (i.e., `connectors_for_plugin_apps(vec![], plugin_apps)` returns entries with connector ID as name, no description/logo)

**In `codex-rs/app-server/src/config_api.rs` tests (or `codex-rs/app-server/tests/`):**
- Add a test asserting that `configRequirements/read` returns `None` when `CloudRequirementsLoader::default()` is used

**In `codex-rs/core/src/connectors.rs` or `connectors_tests.rs`:**
- Add a test for `merge_connectors_with_accessible(vec![], mcp_apps, false)` → returns all MCP apps
- Add a test for `connectors_for_plugin_apps(vec![], plugin_apps)` → returns synthetic entries

```bash
git add codex-rs/
git commit -m "Add regression tests for MCP-only apps, synthetic plugin summaries, and null configRequirements"
```

- [ ] **Step 2: Full build**

```bash
cd codex-rs && cargo build 2>&1 | tail -20
```
Expected: Clean build

- [ ] **Step 3: Run targeted tests**

```bash
cd codex-rs && cargo test -p orbit-code-core 2>&1 | tail -20
cd codex-rs && cargo test -p orbit-code-cli 2>&1 | tail -20
cd codex-rs && cargo test -p orbit-code-app-server 2>&1 | tail -20
cd codex-rs && cargo test -p orbit-code-exec 2>&1 | tail -20
cd codex-rs && cargo test -p orbit-code-tui 2>&1 | tail -20
cd codex-rs && cargo test -p orbit-code-tui-app-server 2>&1 | tail -20
```
Expected: All pass

- [ ] **Step 4: Check TUI snapshots**

```bash
cargo insta pending-snapshots -p orbit-code-tui
cargo insta pending-snapshots -p orbit-code-tui-app-server
```

Review and accept any snapshot changes caused by connector/app UI differences.

- [ ] **Step 5: Verify no stale references**

```bash
# Check for Rust imports of removed crates
grep -rn 'orbit_code_chatgpt\|orbit_code_backend_client\|orbit_code_cloud_tasks\|orbit_code_cloud_requirements\|orbit_code_connectors\|orbit_code_responses_api_proxy\|orbit_code_backend_openapi' \
  codex-rs/ --include='*.rs' --include='*.toml' | grep -v target/ | head -10

# Check for stale Cargo.toml dependency entries
grep -rn 'orbit-code-chatgpt\|orbit-code-backend-client\|orbit-code-cloud-tasks\|orbit-code-cloud-requirements\|orbit-code-connectors\|orbit-code-responses-api-proxy\|orbit-code-backend-openapi' \
  codex-rs/ --include='Cargo.toml' | grep -v target/ | head -10
```
Expected: No output for both

- [ ] **Step 6: Verify directories deleted**

```bash
for crate in chatgpt backend-client codex-backend-openapi-models connectors responses-api-proxy cloud-requirements cloud-tasks cloud-tasks-client; do
  if [ -d "codex-rs/$crate" ]; then
    echo "STILL EXISTS: $crate"
  fi
done
```
Expected: No output

- [ ] **Step 7: Ask user before running full workspace test suite**

This is a large shared-surface change. Ask the user before running `cargo test` across the full workspace.

- [ ] **Step 8: Final commit**

```bash
git add -A
git commit -m "Stage 2 complete: removed 8 OpenAI-only crates, clean build verified"
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

### Config cleanup
- **`chatgpt_base_url` config field** — Dead after removing all chatgpt/cloud-requirements callers. Remove from config types.

### Code cleanup
- **`local_chatgpt_auth.rs` in tui_app_server** — Module name references "chatgpt" but only depends on `core::auth` and `app-server-protocol`. Consider renaming.
- **Auth-refresh cloud requirements rebuild** — The app-server auth-refresh path in `orbit_code_message_processor.rs` currently rebuilds the cloud requirements loader after login/logout. After this change it just uses `CloudRequirementsLoader::default()` again, which is a no-op. The rebuild code can be simplified to remove the dead `cloud_requirements` variable threading. This is safe to do in a follow-up because the current code still compiles and runs correctly — it just passes around a default value that does nothing.

### Feature replacements
- **Rate limit display** — Stubbed to empty. Implement against Orbit backend when available.
- **Enterprise managed config** — `CloudRequirementsLoader` type is ready if Orbit needs managed requirements. Build a new provider when needed.
- **App directory** — Currently MCP-only. If Orbit needs a centralized app directory, implement a provider-neutral directory service.

### Client-facing semantics
- **`configRequirements/read` returns `null`** — This means "no managed requirements configured." Downstream clients that currently treat `null` as "no requirements" will behave correctly. Clients that distinguish "no requirements configured" from "managed requirements feature removed" should be identified and updated if any exist. The app-server API docs updated in Task 6 cover this.
- **`tool_suggest` installable connectors** — Without a directory provider, `list_tool_suggest_discoverable_tools_with_auth` only returns MCP-discovered tools. Directory-only installable connectors no longer appear in tool suggestions. If Orbit needs installable connector suggestions, a new directory provider must be added.
- **Synthetic plugin app metadata** — Plugin apps now get ID-only synthetic entries (connector ID as name, no description/logo/install URL). Orbit clients displaying plugin metadata to users may want copy changes to handle missing fields gracefully (e.g., hide description area when empty, use a generic placeholder icon).
