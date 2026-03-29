# Upstream Sync Analysis

> **Generated:** 2026-03-29
> **Fork point:** `01df50cf4` (2026-03-18, ~0.116.0-alpha.11 + 7)
> **Upstream HEAD:** `38e648ca6` on `upstream/main`
> **Latest upstream release:** `rust-v0.118.0-alpha.3` (latest-alpha-cli)
> **Delta:** 292 commits, 2328 files changed upstream

---

## Executive Summary

The upstream repo has undergone **massive structural refactoring** since our fork. The single biggest change is the **merger of `tui_app_server` into `tui`** (the legacy TUI split was removed). Additionally, **25 new crates** were extracted from `core` and other locations, and several features were deleted entirely.

Our fork has **995 commits** with heavy modifications: the `codex → orbit-code` rename, Anthropic Claude provider integration, thinking tokens in TUI, permission modes work, and `.codex → .orbit` config directory migration.

### The Three Buckets

| Bucket | Files | Risk | Action |
|--------|-------|------|--------|
| **Upstream-only** (no conflict) | 1,644 | LOW | Can `git checkout upstream/main -- <file>` directly |
| **Both changed** (conflict zone) | 684 | HIGH | Must merge manually, file by file |
| **Ours-only** (no conflict) | 2,349 | NONE | Our additions, safe |

---

## CRITICAL: Structural Breaking Changes

### 1. `tui_app_server` → `tui` Rename (BIGGEST CHANGE)

**Upstream commit:** `61429a6c1` — "Rename tui_app_server to tui (#16104)"
**Preceded by:** `d65deec61` — "Remove the legacy TUI split (#15922)"

Upstream **deleted `tui_app_server/` entirely** (1,084 files deleted) and consolidated everything into `tui/`. The standalone `tui` (which used `orbit-code-core` directly) was removed — now only the app-server-backed TUI exists, and it lives at `codex-rs/tui/`.

**Impact on us:** We still have both `tui/` and `tui_app_server/` as separate crates. We've made changes to both. We need to decide:
- Option A: Follow upstream's merge (delete `tui_app_server`, move everything into `tui`). Requires replaying our TUI changes onto the merged crate.
- Option B: Keep both crates for now, cherry-pick individual fixes from upstream's TUI work. More surgical but we diverge further from upstream structure.

**Recommendation:** Option B for now. Cherry-pick bug fixes and features from the upstream TUI into our two crates. Plan the merge as a separate tracked task.

### 2. Massive Crate Extraction from `core`

Upstream extracted **12+ sub-crates** from `codex-rs/core/`. This is why the core conflict zone is 260 files — much of what was in `core` has been moved out:

| New Crate | Extracted From | Purpose |
|-----------|---------------|---------|
| `core-skills` | `core/src/skills/` | System skills (image gen, plugin creator) |
| `instructions` | `core/src/instructions/` | Instruction loading/resolution |
| `sandboxing` | `core/src/sandbox/` | Sandbox policy, transforms, builders |
| `analytics` | `core/src/analytics/` | Usage analytics |
| `plugin` | `core/src/plugins/` | Plugin management |
| `rollout` | `core/src/rollout/` | Feature rollout flags |
| `features` | `core/src/features/` | Feature flag definitions |
| `git-utils` | `core/src/git/` | Git utility functions |
| `terminal-detection` | `core/src/terminal/` | Terminal type detection |
| `connectors` | `core/src/connectors/` | API connectors/clients |
| `tools` | `core/src/tools/` | Tool spec extraction, named tools |
| `code-mode` | `core/src/code_mode/` | V8 code execution mode |

**Impact on us:** Our `core` crate still has all this code inline. When we take upstream changes to `core`, many imports will point to crates that don't exist in our repo. We need to either:
- Add these crates (complex, may conflict with our renames)
- Keep our monolithic `core` and manually port the logic changes without the structural refactor

**Recommendation:** Do NOT extract crates now. Port individual bug fixes and feature logic from upstream's split crates into our monolithic `core`. Track the structural alignment as a future task.

### 3. Artifacts Crate Dropped

**Upstream commits:**
- `b00a05c78` — "feat: drop artifact tool and feature (#15851)"
- `6dcac41d5` — "chore: drop artifacts lib (#15864)"

Upstream completely removed the artifacts system. We still have `codex-rs/artifacts/` as a workspace member.

**Impact:** If we're not using artifacts, we can drop it too. If we are, we're diverged and on our own.

### 4. Custom Prompts Removed

**Upstream commit:** `48144a7fa` — "Remove remaining custom prompt support (#16115)"

### 5. Voice Transcription Removed

**Upstream commit:** `3bbc1ce00` — "Remove TUI voice transcription feature (#16114)"

### 6. `environment` Crate Likely Merged

Upstream doesn't have an `environment` crate anymore — it was folded into `exec-server`. We still have it.

### 7. Template Engine Replacement

**Upstream commit:** `7ef3cfe63` — "feat: replace askama by custom lib (#15784)"

Introduced `codex-utils-template` to replace askama. Multiple follow-up commits migrate templates.

---

## New Upstream Crates (25 total)

### Must-Have (core functionality)

| Crate | Purpose | Complexity to Add |
|-------|---------|-------------------|
| `sandboxing` | Sandbox policy, transforms, builders | HIGH — extracted from our core |
| `features` | Feature flag definitions | MEDIUM |
| `rollout` | Feature rollout system | MEDIUM |
| `analytics` | Usage analytics | LOW |
| `login` (refactored) | Auth moved from core to login | MEDIUM |
| `core-skills` | System skills extraction | LOW |
| `instructions` | Instruction loading | LOW |

### Nice-to-Have (new features)

| Crate | Purpose | Complexity to Add |
|-------|---------|-------------------|
| `plugin` | Plugin management system | HIGH |
| `tools` | Tool spec extraction/named tools | HIGH |
| `code-mode` | V8-powered code execution | HIGH |
| `connectors` | API connector abstractions | MEDIUM |
| `terminal-detection` | Terminal type detection | LOW |
| `git-utils` | Git utility functions | LOW |

### Skip for Now

| Crate | Reason |
|-------|--------|
| `v8-poc` | Experimental, not ready |
| `backend-client` | OpenAI-specific backend client |
| `cloud-requirements` | OpenAI cloud infrastructure |
| `cloud-tasks` / `cloud-tasks-client` | OpenAI cloud tasks |
| `codex-backend-openapi-models` | OpenAI API models |
| `responses-api-proxy` | OpenAI Responses API proxy |

### New Utility Crates

| Crate | Purpose | Take? |
|-------|---------|-------|
| `utils/template` | Custom template engine (replaces askama) | YES if upstream templates are needed |
| `utils/plugins` | Plugin utility functions | YES if taking plugin system |
| `utils/path-utils` | Path manipulation | YES |
| `utils/output-truncation` | Output truncation helpers | YES |

---

## Conflict Zone: 684 Files by Risk Level

### EXTREME RISK (260 files) — `codex-rs/core/`

We have heavy modifications to core (Claude provider, thinking tokens, permission modes). Upstream has massive refactoring (crate extractions, multi-agent v2, guardian changes).

**Key conflict files and what changed:**

| File | Our Change | Upstream Change |
|------|-----------|-----------------|
| `core/src/codex.rs` | +471/-360 (Claude agent loop) | Multi-agent v2, fork snapshots, rollback context |
| `core/src/client.rs` | +302/-99 (Anthropic client) | Connector extraction, auth refactor |
| `core/src/exec.rs` | +50/-34 (exec changes) | Exec server split, sandbox extraction |
| `core/Cargo.toml` | +78/-64 (orbit-code renames) | 12+ new crate dependencies |
| `core/src/error.rs` | +14/- (custom errors) | Protocol error changes |
| `core/src/config/*` | Permission modes | Network proxy, managed features |
| `core/src/guardian/*` | — | Follow-up reminders, denial rationale |
| `core/src/agent/*` | — | Multi-agent v2 path addressing |
| `core/src/exec_policy*` | — | Policy refactoring |
| `core/src/hook_runtime.rs` | — | Non-streaming PreToolUse/PostToolUse |

**Strategy:** Cherry-pick individual logic changes. Do NOT take wholesale file replacements.

### HIGH RISK (62+82 files) — `codex-rs/tui/` + `codex-rs/tui_app_server/`

**TUI changes worth taking:**
- `/title` terminal title configuration (`60cd0cf75`)
- `/compact` follow-up queuing (`e838645fa`)
- Shift+Left tmux edit (`5e3793def`)
- Plugin menu (`f7201e5a9`, `b5d0a5518`)
- Fix duplicate /review messages (`0d44bd708`)
- Collaboration mode footer refresh (`bede1d9e2`)
- Agent picker fixes (`ed977b42a`, `46b653e73`)
- Stale turn steering fix (`c0ffd000d`)
- Skills toggle/picker fixes

**TUI changes to SKIP:**
- Voice transcription (removed upstream anyway)
- Plugin install/uninstall UI (depends on plugin crate)
- ChatGPT login flows (we use Anthropic)
- App-server originator workarounds (upstream-specific)

### HIGH RISK (35+52 files) — `app-server-protocol/` + `app-server/`

**Worth taking:**
- v2 protocol additions (thread fork, rollback, metadata update)
- MCP startup status notifications
- Filesystem watch support
- Back-pressure and batching for command/exec
- WebSocket auth

**Skip:**
- ChatGPT device-code login
- Business plan types
- Feature flag override method

### MEDIUM RISK (5 files) — `codex-rs/protocol/`

Protocol is foundational — changes here cascade everywhere.

| File | Upstream Change |
|------|-----------------|
| `protocol.rs` | New Op variants for multi-agent v2, spawn v2 |
| `models.rs` | New model types |
| `permissions.rs` | Permission changes |
| `openai_models.rs` | OpenAI-specific model updates |

**Strategy:** Review each change individually. Take Op/EventMsg additions that don't break our existing protocol usage.

### LOW RISK — Other Conflict Zones

| Area | Files | Notes |
|------|-------|-------|
| `hooks/` | 8 | Non-streaming PreToolUse/PostToolUse — likely want this |
| `cli/` | 5 | CLI argument changes |
| `exec/` | 8 | Exec refactoring |
| `login/` | 5 | Auth code moved here from core |
| `config/` | 2 | Config type changes |
| `.github/` | 17 | CI workflow changes |

---

## Upstream-Only: 1,644 Files — Safe to Take

These files were only changed upstream and we haven't touched them. They can be taken directly with `git checkout upstream/main -- <path>`.

### By Area (top items)

| Area | Files | Action |
|------|-------|--------|
| `tui_app_server/` | 1,002 | **DON'T TAKE** — this is the old crate that was deleted upstream |
| `tui/` | 122 | Take selectively (new features, bug fixes) |
| `core/` | 94 | Take carefully (new files from extraction) |
| `app-server-protocol/` | 56 | Take new schema files |
| `utils/` | 34 | Take new utility crates |
| `tools/` (codex-rs) | 28 | Take if adding tools crate |
| `exec-server/` | 20 | Take selectively |
| `core-skills/` | 18 | Take if adding core-skills crate |
| `skills/` | 17 | Take new skills |
| `sandboxing/` | 16 | Take if adding sandboxing crate |
| `rollout/` | 15 | Take if adding rollout crate |
| `argument-comment-lint` | 13 | Low priority |
| `hooks/` | 13 | Take hook improvements |
| `login/` | 12 | Take auth refactoring |
| `protocol/` | 11 | Take new protocol types |
| `git-utils/` | 11 | Take if adding git-utils crate |
| `code-mode/` | 11 | Skip (V8 integration) |
| `app-server/` | 10 | Take selectively |

---

## Recommended Sync Strategy

### Phase 1: Safe Cherry-Picks (no structural changes)

Take bug fixes and small features that don't depend on new crate extractions:

```
# Hooks improvements
73bbb07ba — PreToolUse non-streaming support
c4d9887f9 — PostToolUse non-streaming support
267499bed — prompt continuation user message

# TUI bug fixes
bede1d9e2 — collaboration mode footer refresh
0d44bd708 — fix duplicate /review messages
ed977b42a — agent picker regression fix

# Sandbox fixes
ec089fd22 — bwrap lookup for multi-entry PATH
937cb5081 — old system bubblewrap compatibility
b6050b42a — resolve bwrap from trusted PATH
86764af68 — first-time project .codex creation protection
d76124d65 — MACOS_DEFAULT_PREFERENCES_POLICY fix

# Protocol additions (review individually)
# TUI features
60cd0cf75 — /title terminal title config
e838645fa — /compact follow-up queuing
5e3793def — Shift+Left tmux edit
```

### Phase 2: New Utility Crates (low risk additions)

Add new utility crates that don't conflict with existing code:

```
codex-rs/utils/template/       — custom template engine
codex-rs/utils/path-utils/     — path manipulation
codex-rs/utils/output-truncation/ — output truncation
codex-rs/git-utils/            — git utility functions
codex-rs/terminal-detection/   — terminal type detection
```

### Phase 3: Core Logic Ports (high effort)

Manually port logic changes from upstream's refactored crates into our monolithic core:

- Guardian follow-up reminders and denial rationale threading
- Multi-agent v2 communication (if needed)
- Fork snapshot modes
- Network proxy refactoring
- Auth code reorganization

### Phase 4: Structural Alignment (future)

- Merge `tui_app_server` into `tui` (matching upstream)
- Extract sub-crates from `core` to match upstream structure
- Add plugin system
- Add features/rollout crates

---

## Files Changed Upstream — Full Categorized List

### New Files Added Upstream (255 total)

```
codex-rs/tools/          — 27 new files (tool spec extraction)
codex-rs/core/           — 27 new files (new modules before extraction)
codex-rs/tui/            — 23 new files (new TUI features)
codex-rs/app-server-protocol/ — 21 new files (v2 schemas)
codex-rs/skills/         — 17 new files (new system skills)
codex-rs/utils/          — 15 new files (new utility crates)
codex-rs/exec-server/    — 13 new files (exec split)
codex-rs/sandboxing/     — 11 new files (sandbox extraction)
codex-rs/code-mode/      — 11 new files (V8 code mode)
codex-rs/core-skills/    — 6 new files
codex-rs/plugin/         — 6 new files
codex-rs/hooks/          — 6 new files
codex-rs/instructions/   — 4 new files
codex-rs/rollout/        — 4 new files
codex-rs/login/          — 4 new files
codex-rs/v8-poc/         — 4 new files
third_party/v8/          — 3 new files
```

### Files Deleted Upstream (1,174 total)

```
codex-rs/tui_app_server/ — 1,084 files (RENAMED to tui)
codex-rs/core/           — 35 files (moved to extracted crates)
codex-rs/package-manager/— 11 files (status unclear)
codex-rs/artifacts/      — 11 files (DROPPED)
shell-tool-mcp/          — 14 files (restructured)
codex-rs/tui/            — 4 files (legacy split removed)
codex-rs/test-macros/    — 3 files (status unclear)
codex-rs/environment/    — 2 files (merged into exec-server)
```

---

## Crate Dependency Changes

### Our Workspace Has, Upstream Doesn't

| Crate | Status |
|-------|--------|
| `anthropic` | **OUR ADDITION** — Claude provider, keep |
| `environment` | Upstream removed — merged into exec-server |
| `artifacts` | Upstream dropped — consider removing |
| `package-manager` | Upstream may have removed |
| `test-macros` | Upstream may have removed |
| `tui_app_server` | Upstream renamed to tui |
| `utils/git` | Upstream renamed to `git-utils` top-level crate |

### Upstream Has, We Don't

| Crate | Priority |
|-------|----------|
| `sandboxing` | HIGH — sandbox logic we'll need |
| `features` | MEDIUM — feature flags |
| `rollout` | MEDIUM — feature rollout |
| `analytics` | LOW — usage analytics |
| `core-skills` | LOW — skill extraction |
| `instructions` | LOW — instruction loading |
| `plugin` | FUTURE — plugin system |
| `tools` | FUTURE — tool extraction |
| `code-mode` | SKIP — V8 integration |
| `connectors` | MEDIUM — API connectors |
| `terminal-detection` | LOW |
| `git-utils` | LOW (we have utils/git) |
| `backend-client` | SKIP — OpenAI-specific |
| `cloud-*` | SKIP — OpenAI cloud infra |
| `responses-api-proxy` | SKIP — OpenAI Responses API |
| `v8-poc` | SKIP — experimental |
| `utils/template` | MEDIUM — template engine |
| `utils/plugins` | FUTURE — with plugin system |
| `utils/path-utils` | LOW |
| `utils/output-truncation` | LOW |

---

## How to Execute Cherry-Picks

```bash
# For individual commits (bug fixes, small features):
git cherry-pick -n <commit-hash>  # Stage without committing
# Review changes, resolve conflicts, commit

# For entire new files/crates (upstream-only):
git checkout upstream/main -- codex-rs/<new-crate>/
# Then rename codex- prefixes to orbit-code- in Cargo.toml

# For checking what a specific upstream commit changed:
git show --stat <commit-hash>
git show <commit-hash> -- <specific-file>

# For comparing a specific file between fork and upstream:
git diff 01df50cf4..upstream/main -- codex-rs/core/src/codex.rs
```
