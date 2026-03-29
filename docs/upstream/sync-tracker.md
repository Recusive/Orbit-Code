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
| Phase 1C | New feature crates | ~40 | TODO |
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

**Commit status:** Not yet committed

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

---

## Blocked / Waiting

_None currently._

---

## Next Up

### Phase 1C — New Feature Crates [TODO]

These crates have more internal dependencies and need careful ordering:

| Crate | Package Name | Blocked By |
|-------|-------------|------------|
| `utils/plugins` | `orbit-code-utils-plugins` | Nothing |
| `plugin` | `orbit-code-plugin` | `utils/plugins` |
| `features` | `orbit-code-features` | Nothing |
| `rollout` | `orbit-code-rollout` | `features` |
| `instructions` | `orbit-code-instructions` | Nothing |
| `core-skills` | `orbit-code-core-skills` | `core` changes |
| `sandboxing` | `orbit-code-sandboxing` | `core` changes |

**Note:** `core-skills` and `sandboxing` were extracted FROM `core` upstream. They depend on types that still live in our monolithic `core`. These may need stub implementations or may need to wait until Phase 3-5 when we merge core changes.

---

## Decisions

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-29 | Keep `tui_app_server` separate (don't follow upstream merge into `tui`) | Too risky to restructure during sync. Do as separate task later. |
| 2026-03-29 | Inline missing helpers in `output-truncation` instead of syncing deps first | Lets Phase 1A complete without touching existing crates. Fix forward in Phase 2-3. |
| 2026-03-29 | Skip `code-mode`, `v8-poc`, `backend-client`, `cloud-*` crates | OpenAI-specific or experimental. Not needed for Orbit Code. |
