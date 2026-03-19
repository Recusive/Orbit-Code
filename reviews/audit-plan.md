# Plan Audit: Stage 2 — Remove Dead Crates

**Date**: 2026-03-19
**Plan Document**: `docs/migration/02-remove-dead-crates.md`
**Branch**: main

## Plan Summary

The plan proposes removing 8 OpenAI-specific crates from the Rust workspace in bottom-up order: delete leaf crates first, then mid-level crates, then the most-connected `cloud-requirements` crate. For `cloud-requirements`, callers switch to passing `None` instead of a real loader. The stated goal is removing ~15,000 lines of OpenAI-specific code to create a clean foundation for multi-provider support.

## Files Reviewed

| File | Role | Risk |
|------|------|------|
| `codex-rs/Cargo.toml` | Workspace root — members + dependencies | High |
| `codex-rs/cli/src/main.rs` | CLI dispatcher — subcommand imports | High |
| `codex-rs/cli/Cargo.toml` | CLI dependencies | Medium |
| `codex-rs/app-server/Cargo.toml` | App-server dependencies | High |
| `codex-rs/app-server/src/lib.rs` | App-server bootstrap — cloud_requirements usage | High |
| `codex-rs/app-server/src/orbit_code_message_processor.rs` | Agent message handler — chatgpt, backend-client, connectors usage | High |
| `codex-rs/exec/Cargo.toml` | Exec dependencies | Medium |
| `codex-rs/exec/src/lib.rs` | Exec — cloud_requirements usage | Medium |
| `codex-rs/tui/Cargo.toml` | TUI dependencies | High |
| `codex-rs/tui/src/lib.rs` | TUI bootstrap — cloud_requirements, chatgpt_base_url | High |
| `codex-rs/tui/src/chatwidget.rs` | TUI chat — BackendClient, connectors::AppInfo | High |
| `codex-rs/tui_app_server/Cargo.toml` | TUI app-server dependencies | High |
| `codex-rs/tui_app_server/src/lib.rs` | TUI app-server bootstrap — cloud_requirements | High |
| `codex-rs/core/Cargo.toml` | Core engine dependencies | High |
| `codex-rs/core/src/connectors.rs` | Core connectors — orbit_code_connectors imports | High |
| `codex-rs/chatgpt/src/lib.rs` | ChatGPT crate — connectors module definition | Medium |
| `pnpm-workspace.yaml` | pnpm workspace — responses-api-proxy entry | Low |

_Risk: High (core logic, many dependents), Medium (feature code), Low (utilities, tests)_

## Verdict: NEEDS REWORK

The plan has an **incorrect dependency graph** that will cause build failures. Three crates listed as "leaf" or "dependents already removed" have undiscovered live dependents, and the scope of source code changes is significantly underestimated for 4 crates (tui, tui_app_server, core, app-server).

---

## Critical Issues (Must Fix Before Implementation)

| # | Section | Problem | Recommendation |
|---|---------|---------|----------------|
| 1 | 2.1 | **`connectors` crate is NOT a leaf** — `core/Cargo.toml` depends on `orbit-code-connectors`. `core/src/connectors.rs` imports `AllConnectorsCacheKey`, `DirectoryListResponse`, `CONNECTORS_CACHE_TTL`, and `list_all_connectors_with_options()`. Removing `connectors` without updating `core` will break the build. | Move `connectors` from Task 1 (leaf) to Task 2 (mid-level). Add `core/Cargo.toml` and `core/src/connectors.rs` edits: replace `orbit_code_connectors` imports with stubs or inline the types. |
| 2 | 2.1 | **`backend-client` has live dependents** — `tui/Cargo.toml` and `app-server/Cargo.toml` both depend on `orbit-code-backend-client`. `tui/src/chatwidget.rs` uses `BackendClient::from_auth()` for rate limits; `app-server/src/orbit_code_message_processor.rs` does the same. Plan says "EASY — dependents already removed" — wrong. | Add `tui/Cargo.toml`, `tui/src/chatwidget.rs`, `app-server/Cargo.toml`, `app-server/src/orbit_code_message_processor.rs` to the modification list. Stub rate limit functions to return empty vecs. |
| 3 | 2.1 | **TUI source changes completely missing for `cloud-requirements`** — Plan says "(The TUI may not directly call the loader — check if there are source imports too)". Reality: `tui/src/lib.rs` has 15+ references to `cloud_requirements_loader`, `CloudRequirementsLoader` type, and `.cloud_requirements()` builder calls. | Add full `tui/src/lib.rs` edits to Task 3: remove import, replace all `cloud_requirements_loader(...)` calls with `CloudRequirementsLoader::default()` or `None`, update function signatures. |
| 4 | 2.1 | **TUI and tui_app_server deps on `chatgpt` not addressed** — Both `tui/Cargo.toml` and `tui_app_server/Cargo.toml` depend on `orbit-code-chatgpt`. Used for `connectors::AppInfo` type in chatwidget, skills, composer, app_event, and extensive tests (~30 test references each). Plan only removes chatgpt from cli and app-server. | Add TUI and tui_app_server to Task 2. Replace `orbit_code_chatgpt::connectors::AppInfo` imports with `orbit_code_core::connectors::AppInfo` (already re-exported from `orbit_code_app_server_protocol`). |
| 5 | 2.1 | **Wrong file name** — Plan references `codex-rs/app-server/src/codex_message_processor.rs`. Actual file is `codex-rs/app-server/src/orbit_code_message_processor.rs` (renamed in Stage 1). | Fix the file name in the plan. |

## Recommended Improvements (Should Consider)

| # | Section | Problem | Recommendation |
|---|---------|---------|----------------|
| 1 | 2.5 | **`core/src/connectors.rs` needs deep surgery** — This file calls `orbit_code_connectors::list_all_connectors_with_options()` which does HTTP calls to the ChatGPT backend. Removing the connectors crate requires either inlining the function or stubbing it to return empty results. The `AllConnectorsCacheKey` and `DirectoryListResponse` types also need stubs. | Create a minimal stub in `core` or return empty results from the connector listing functions. This is the hardest part of removing `connectors` — document the approach explicitly. |
| 2 | 2.5 | **`git add -A` in commit steps** — Multiple tasks use `git add -A` which may accidentally stage untracked files (secrets, build artifacts, IDE config). | Use `git add` with specific paths or at minimum `git add codex-rs/ pnpm-workspace.yaml` to scope the staging. |
| 3 | 2.2 | **`chatgpt_base_url` config field becomes dead** — After removing cloud-requirements and chatgpt, the `chatgpt_base_url` field in config is no longer used by any production code path. It should be cleaned up or at least noted for future removal. | Add a follow-up note or TODO to remove the `chatgpt_base_url` config field in a later stage. |
| 4 | 2.5 | **`local_chatgpt_auth.rs` in tui_app_server** — This module (`tui_app_server/src/local_chatgpt_auth.rs`) has "chatgpt" in the name but only depends on `core::auth` and `app-server-protocol`. It does NOT depend on the `chatgpt` crate. No action needed but worth confirming during implementation. | Verify it compiles without `orbit-code-chatgpt` dep. Consider renaming in a future cleanup. |
| 5 | 2.8 | **Test scope is underestimated** — Plan only runs `cargo test -p orbit-code-core` and `cargo test -p orbit-code-cli`. Both TUI crates have extensive connector/chatgpt tests that will break. | Run `cargo test --workspace` or at minimum add `-p orbit-code-tui -p orbit-code-tui-app-server -p orbit-code-app-server` to the test commands. |
| 6 | 2.5 | **No `cargo check` between Task 1 and Task 2** — Task 1 removes leaf crates but doesn't verify the build before proceeding. If the dependency graph is wrong (as found), errors cascade. | Add `cargo check` after every task, not just Tasks 2 and 3. |

## Nice-to-Haves (Optional Enhancements)

| # | Section | Idea | Benefit |
|---|---------|------|---------|
| 1 | 2.6 | Run `cargo-shear` after removal to detect any newly-unused deps | Catches transitive dependencies that are no longer needed after removing 8 crates |
| 2 | 2.6 | Update `[workspace.metadata.cargo-shear] ignored` list | Some ignored entries may no longer be needed |
| 3 | 2.2 | Clean up `[patch.crates-io]` entries that reference OpenAI forks | Future cleanup opportunity — not blocking |

## Edge Cases Not Addressed

- **What happens if `core/src/connectors.rs` functions are called at runtime after the `connectors` crate is removed?** The plan doesn't address the `core` dependency on `connectors` at all. `list_all_connectors_with_options()` in `core/src/connectors.rs` calls into the `connectors` crate to fetch connector data from the ChatGPT backend API. Removing the crate without updating this function will fail to compile.

- **What happens to the `connectors_tests.rs` in core?** `core/src/connectors_tests.rs` uses `CloudRequirementsLoader::new(async move { ... })` directly. After removing `cloud-requirements`, this test file needs updating to use the default/None loader pattern.

- **What about `orbit-code-chatgpt::connectors::AppInfo` vs `orbit_code_app_server_protocol::AppInfo`?** The `AppInfo` type is defined in `app-server-protocol` and re-exported through `core::connectors`. The `chatgpt::connectors` module provides *different* functions (like `list_apps`, `connector listing via ChatGPT API`). Simply re-pointing imports from `chatgpt::connectors::AppInfo` to `core::connectors::AppInfo` should work for the type, but the connector *functions* need stubs.

- **What happens to Bazel `BUILD.bazel` files?** The `backend-client/BUILD.bazel` file exists. If Bazel builds are used in CI, they'll also need updating. The plan only addresses Cargo.

- **What happens if the `cloud-tasks` member uses a direct path dep instead of workspace?** The CLI references `cloud-tasks` with `orbit-code-cloud-tasks = { path = "../cloud-tasks" }` (not `workspace = true`), and there's no workspace dependency entry for it. The plan says "Remove from workspace.dependencies" but there may be nothing to remove for this crate.

## Code Suggestions

### Critical Issue 1: Fix core's dependency on connectors

In `codex-rs/core/Cargo.toml`, remove:
```toml
orbit-code-connectors = { workspace = true }
```

In `codex-rs/core/src/connectors.rs`, replace the `orbit_code_connectors` imports. The simplest approach — inline minimal type stubs and return empty results:

```rust
// BEFORE:
use orbit_code_connectors::AllConnectorsCacheKey;
use orbit_code_connectors::DirectoryListResponse;
pub use orbit_code_connectors::CONNECTORS_CACHE_TTL;

// AFTER:
// Stub types — connectors crate removed in Stage 2
pub const CONNECTORS_CACHE_TTL: Duration = Duration::from_secs(300);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct AllConnectorsCacheKey { /* fields */ }

// Stub the list_all_connectors function to return empty
```

The `list_all_connectors_with_options()` call site (line ~435) needs to be replaced with a function that returns an empty result, since the ChatGPT connector directory API is no longer accessible.

### Critical Issue 2: Fix backend-client dependents

In `codex-rs/tui/src/chatwidget.rs`, replace:
```rust
// BEFORE:
use orbit_code_backend_client::Client as BackendClient;

// AFTER (in fetch_rate_limits function):
async fn fetch_rate_limits(_base_url: String, _auth: CodexAuth) -> Vec<RateLimitSnapshot> {
    Vec::new() // Rate limits require OpenAI backend — stub until Orbit backend supports this
}
```

Same pattern in `app-server/src/orbit_code_message_processor.rs`.

### Critical Issue 3: Fix TUI cloud_requirements

In `codex-rs/tui/src/lib.rs`, replace all usages:
```rust
// BEFORE:
use orbit_code_cloud_requirements::cloud_requirements_loader;
// ...
let cloud_requirements = cloud_requirements_loader(
    cloud_auth_manager,
    chatgpt_base_url,
    orbit_code_home.to_path_buf(),
);

// AFTER:
use orbit_code_core::config_loader::CloudRequirementsLoader;
// ...
let cloud_requirements = CloudRequirementsLoader::default();
```

### Critical Issue 4: Fix chatgpt::connectors imports

In all TUI files (`tui/src/chatwidget.rs`, `tui/src/app_event.rs`, `tui/src/bottom_pane/chat_composer.rs`, `tui/src/chatwidget/skills.rs`, and their `tui_app_server` equivalents):

```rust
// BEFORE:
use orbit_code_chatgpt::connectors;
use orbit_code_chatgpt::connectors::AppInfo;

// AFTER:
use orbit_code_core::connectors;
use orbit_code_app_server_protocol::AppInfo;
```

---

## Verdict Details

### Correctness: CONCERNS

The dependency graph in the plan is factually wrong for 3 of 8 crates:
1. `connectors` — has undiscovered dependent: `core`
2. `backend-client` — has undiscovered dependents: `tui`, `app-server`
3. `chatgpt` — has undiscovered dependents: `tui`, `tui_app_server`

Executing the plan as written will fail at `cargo check` after Task 1 (connectors removal breaks core) and again after Task 2 (backend-client removal breaks tui and app-server).

### Architecture: PASS

The overall approach (bottom-up removal, passing None for cloud requirements) is sound. The `CloudRequirementsLoader` type surviving in `core::config_loader` while implementations are removed is the right pattern. The problem is purely in the dependency analysis, not the architectural approach.

### Performance: PASS (N/A)

This is a code removal plan. No performance impact — code is being deleted, not added. The stub functions returning empty vecs are O(1) with no allocations.

### Production Readiness: CONCERNS

- The `fetch_rate_limits` function in TUI and app-server will silently return empty results after stubbing. This should be documented so users know rate limit display is temporarily unavailable.
- The connector listing functions in `core` will need careful stubbing to ensure the apps/connectors feature degrades gracefully rather than erroring.
- The plan's verification step (Task 5, Step 5) only greps for Rust import names — it doesn't check `Cargo.toml` dependency entries. A stale `[dependencies]` entry pointing to a deleted path will cause build failure.

### Extensibility: PASS

The plan correctly notes that the remaining ~65 crates form a clean foundation for multi-provider support. Removing the OpenAI-specific crates is the right move. The `CloudRequirementsLoader` interface in core provides the extensibility point for future enterprise config systems.
