# Upstream Sync Tracker

> **Started:** 2026-03-29
> **Fork point:** `01df50cf4` (2026-03-18)
> **Target:** `38e648ca6` (upstream/main)
> **Plan:** `docs/tracked/todo/upstream-sync-plan.md`
> **Analysis:** `docs/upstream/sync-analysis-2026-03-29.md`

---

## Progress Overview

| Phase | Description | Files | Status |
|-------|------------|-------|--------|
| Phase 0 | Safety verification | 0 | DONE |
| Phase 1A | New utility crates (5) | 28 | DONE |
| Phase 1B | Wire into workspace | 1 | DONE |
| Phase 1C | New feature crates (8) | 76 | DONE |
| Phase 2 | Upstream-only modified files | ~245 | TODO |
| Phase 3 | Easy conflict files (≤20 lines) | ~339 | TODO |
| Phase 4 | Medium conflict files (21-100 lines) | ~262 | TODO |
| Phase 5 | Hard conflict files (>100 lines) | ~83 | TODO |
| Phase 6 | Verification & cleanup | 0 | TODO |

---

## Completed Work

### Phase 1A — New Utility Crates [DONE 2026-03-29]

| Crate | Package Name | Compiles | Deps Wired | Notes |
|-------|-------------|----------|------------|-------|
| `utils/template` | `orbit-code-utils-template` | PASS | PASS | Zero codex deps, pure std-lib |
| `utils/path-utils` | `orbit-code-utils-path` | PASS | PASS | Dep: `absolute-path` (exists) |
| `utils/output-truncation` | `orbit-code-utils-output-truncation` | PASS | PASS | **DEBT:** inlined helpers, see below |
| `git-utils` | `orbit-code-git-utils` | PASS | PASS | Dep: `absolute-path` (exists) |
| `terminal-detection` | `orbit-code-terminal-detection` | PASS | PASS | No workspace deps |

**Commit:** `e9829bf0d` on `upstream-0.118.0`

### Phase 1C — New Feature Crates [DONE 2026-03-29]

| Crate | Package Name | Compiles | Shims Needed | Notes |
|-------|-------------|----------|-------------|-------|
| `utils/plugins` | `orbit-code-utils-plugins` | PASS | None | No codex deps |
| `plugin` | `orbit-code-plugin` | PASS | None | Deps: utils-plugins, absolute-path |
| `instructions` | `orbit-code-instructions` | PASS | None | Dep: protocol |
| `sandboxing` | `orbit-code-sandboxing` | PASS | Yes (DEBT-002) | macOS PermissionProfile compat |
| `features` | `orbit-code-features` | PASS | None | Deps: login, otel, protocol |
| `analytics` | `orbit-code-analytics` | PASS | Yes (DEBT-003) | Retargeted to orbit-code-core |
| `rollout` | `orbit-code-rollout` | PASS | Yes (DEBT-004) | GitSha→String, dropped Custom variant |
| `core-skills` | `orbit-code-core-skills` | PASS | Yes (DEBT-005) | Config shape adaptation, loader helpers inlined |

---

## Technical Debt

### DEBT-001: output-truncation inlined helpers

**File:** `codex-rs/utils/output-truncation/src/lib.rs`
**Priority:** Medium
**Fix in:** Phase 2-3 (when syncing `utils/string` and `protocol`)

Upstream's `output-truncation` imports these from `codex-utils-string`:
- `approx_token_count()`
- `approx_bytes_for_tokens()`
- `approx_tokens_from_byte_count()`
- `truncate_middle_chars()`
- `truncate_middle_with_token_budget()`

And calls these methods on `TruncationPolicy`:
- `.byte_budget()`
- `.token_budget()`

Our `orbit-code-utils-string` and `orbit-code-protocol` don't have these yet. Agent inlined the logic directly into the crate as a workaround. Works, but duplicates code.

**To fix:** Sync `utils/string` to add missing functions → sync `protocol` to add `TruncationPolicy` methods → refactor `output-truncation/src/lib.rs` to import instead of inline. Compare with `reference/codex-upstream/codex-rs/utils/output-truncation/src/lib.rs`.

### DEBT-002: sandboxing PermissionProfile compat

**File:** `codex-rs/sandboxing/src/policy_transforms.rs`
**Priority:** Medium
**Fix in:** Phase 3-5 (when syncing protocol with upstream PermissionProfile changes)

Added macOS `PermissionProfile` compatibility shim for normalize/merge/intersect operations. Upstream has a different `PermissionProfile` shape.

### DEBT-003: analytics retargeted to orbit-code-core

**File:** `codex-rs/analytics/src/analytics_client.rs`
**Priority:** Low
**Fix in:** Phase 3-5 (when syncing core)

Upstream's analytics imports `create_client` and `originator` from a different module path. Agent retargeted to `orbit-code-core`. Will align when core is synced.

### DEBT-004: rollout compatibility changes

**Files:** `codex-rs/rollout/src/{lib,recorder,metadata,list,policy}.rs`
**Priority:** Medium
**Fix in:** Phase 3-5 (when syncing protocol and state)

- `GitSha` converted to `String` (our protocol doesn't have `GitSha` type yet)
- Dropped `SessionSource::Custom` match arms (variant doesn't exist in our protocol)
- `truncate_middle_chars` sourced from `output-truncation` instead of `utils-string`
- `default_client` retargeted to `orbit-code-core`
- Handled `EventMsg::ListCustomPromptsResponse` variant

### DEBT-005: core-skills config shape adaptation

**Files:** `codex-rs/core-skills/src/{config_rules,loader,model}.rs`
**Priority:** Medium
**Fix in:** Phase 3-5 (when syncing core config)

- Skill config rules simplified to current path-only shape
- Project-root-marker helpers copied inline (not yet public in our core)
- `Product::matches_product_restriction` replaced with direct membership checks

---

## Blocked / Waiting

_None currently._

---

## Next Up

### Phase 2 — Upstream-Only Modified Files [TODO]

245 files that upstream modified but we never touched. Safe to take with rename transformation. Organized by crate layer:

| Layer | Files | Crates |
|-------|-------|--------|
| Foundation | ~20 | protocol, config, hooks, execpolicy |
| Utilities | ~14 | utils/*, rmcp-client, network-proxy, shell-command, apply-patch |
| Core | ~68 | core (new modules + upstream-only modifications) |
| TUI | ~82 | tui (new features, bug fixes) |
| App-server | ~33 | app-server, app-server-protocol |
| Other | ~28 | exec-server, login, mcp-server, skills, windows-sandbox |

---

## Decisions

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-29 | Keep `tui_app_server` separate (don't follow upstream merge into `tui`) | Too risky to restructure during sync. Do as separate task later. |
| 2026-03-29 | Inline missing helpers in `output-truncation` instead of syncing deps first | Lets Phase 1A complete without touching existing crates. Fix forward in Phase 2-3. |
| 2026-03-29 | Skip `code-mode`, `v8-poc`, `backend-client`, `cloud-*` crates | OpenAI-specific or experimental. Not needed for Orbit Code. |
