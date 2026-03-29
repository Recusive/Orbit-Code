# Upstream Sync Plan — Add Only, Never Remove Our Logic

> **Status:** TODO
> **Created:** 2026-03-29
> **Fork point:** `01df50cf4` (2026-03-18)
> **Upstream target:** `38e648ca6` (upstream/main HEAD)
> **Reference clone:** `reference/codex-upstream/`

---

## Golden Rule

**ADD upstream changes. NEVER remove our custom logic.** If a merge would delete or overwrite any of the protected code listed below, stop and merge manually.

---

## Protected Code — DO NOT OVERWRITE

These are our custom additions since the fork. Every phase must preserve them.

### P0 — Core Identity (break these = nothing works)

| Area | Files | What it is |
|------|-------|-----------|
| **Anthropic crate** | `codex-rs/anthropic/src/*.rs` (8 files) | Claude API client, streaming, token refresh |
| **Anthropic auth** | `core/src/anthropic_auth/*.rs` (4 files) | Anthropic OAuth flow |
| **Anthropic bridge** | `core/src/anthropic_bridge.rs` | Anthropic ↔ core bridge |
| **Anthropic model mapping** | `core/src/models_manager/anthropic_mapping*.rs` (2 files) | Claude model → config mapping |
| **Anthropic login** | `login/src/anthropic.rs` | Anthropic login flow |
| **Client modifications** | `core/src/client.rs` (+302 lines) | Multi-provider client routing |
| **Agent loop** | `core/src/codex.rs` (+471 lines) | Claude thinking tokens, provider routing |
| **orbit-code renames** | ALL `Cargo.toml` files, ALL `use` statements | `codex_*` → `orbit_code_*` everywhere |
| **Config dir** | `utils/home-dir/src/lib.rs` | `.codex` → `.orbit`, `CODEX_HOME` → `ORBIT_HOME` |
| **orbit_code_* files** | `core/src/orbit_code_*.rs` (5 files) | Renamed delegate, thread, tests |

### P1 — Our Features (break these = features regress)

| Area | Files | What it is |
|------|-------|-----------|
| **Thinking tokens TUI** | `tui/src/streaming/controller.rs` (+204 lines) | Claude thinking token rendering |
| **Thinking tokens chat** | `tui/src/chatwidget.rs` (+903 lines) | Thinking block display |
| **Permission modes** | `docs/tracked/todo/hooks/permission-modes-*.md` | Accept/Bypass mode work |
| **LMStudio provider** | `codex-rs/lmstudio/src/*.rs` (2 files) | LMStudio integration |
| **Ollama provider** | `codex-rs/ollama/src/*.rs` (2 files) | Ollama integration |
| **Auth flow TUI** | `tui/src/auth_flow*.rs`, `tui/src/bottom_pane/auth_flow_view*.rs` | Custom auth UI |
| **Model presets** | `core/src/models_manager/model_presets.rs` | Anthropic model presets |

### P2 — Our Infrastructure (break these = build fails)

| Area | Files | What it is |
|------|-------|-----------|
| **Environment crate** | `codex-rs/environment/` | Our env abstraction (upstream deleted theirs) |
| **Artifacts crate** | `codex-rs/artifacts/` | Our artifacts (upstream dropped theirs) |
| **Package manager** | `codex-rs/package-manager/` | Our package manager |
| **Test macros** | `codex-rs/test-macros/` | Our test macros |
| **tui_app_server** | `codex-rs/tui_app_server/` | Still separate in our repo |
| **Custom schemas** | `app-server-protocol/schema/json/orbit_code_*.json` | Our renamed schemas |

---

## Rename Transformation Rules

Apply to EVERY file brought in from upstream:

```
# Cargo.toml package names
codex-<name>  →  orbit-code-<name>

# Cargo.toml lib names
codex_<name>  →  orbit_code_<name>

# Rust imports
use codex_<name>::  →  use orbit_code_<name>::

# Workspace dependencies (root Cargo.toml)
codex-<name> = { path = "..." }  →  orbit-code-<name> = { path = "..." }

# Config/env
.codex  →  .orbit
CODEX_HOME  →  ORBIT_HOME
find_codex_home  →  find_orbit_home
codex_home  →  orbit_code_home

# DO NOT RENAME:
# - CODEX_SANDBOX_* env vars (per CLAUDE.md — never touch)
# - codex_tui__ snapshot prefixes (insta test snapshots)
# - Directory names (codex-rs/, codex-api/, codex-client/ stay as-is)
```

---

## Phase 0: Safety Setup

**Goal:** Create an isolated branch with verification infrastructure.

```
Steps:
[ ] 0.1  git checkout -b upstream-sync-phase1
[ ] 0.2  Verify current state compiles: cargo check -p orbit-code-protocol
[ ] 0.3  Run baseline test for core: cargo test -p orbit-code-core --lib -- --test-threads=1 2>&1 | tail -5
[ ] 0.4  Save baseline: git stash list, git status — clean working tree
```

**Checkpoint:** Branch exists, baseline compiles. Abort point if anything is already broken.

---

## Phase 1: New Standalone Crates (Zero Conflict)

**Goal:** Add new utility crates that have NO overlap with our code. Pure additions.

**Risk:** LOW — these are brand new directories.

### 1A — New utility crates

```
Steps:
[ ] 1A.1  Copy codex-rs/utils/template/ from reference/codex-upstream/
[ ] 1A.2  Rename in Cargo.toml: codex-utils-template → orbit-code-utils-template
[ ] 1A.3  Rename lib name: codex_utils_template → orbit_code_utils_template
[ ] 1A.4  Add to workspace members in codex-rs/Cargo.toml
[ ] 1A.5  Add workspace dependency: orbit-code-utils-template = { path = "utils/template" }
[ ] 1A.6  cargo check -p orbit-code-utils-template
[ ] 1A.7  Repeat for: utils/path-utils, utils/output-truncation
```

### 1B — New standalone crates

```
Steps:
[ ] 1B.1  Copy codex-rs/git-utils/ from reference
[ ] 1B.2  Rename package: codex-git-utils → orbit-code-git-utils
[ ] 1B.3  Rename all use codex_* → orbit_code_* in source files
[ ] 1B.4  Add to workspace + workspace deps
[ ] 1B.5  cargo check -p orbit-code-git-utils
[ ] 1B.6  Repeat for: terminal-detection
```

### 1C — New feature crates (no core dependency conflicts)

```
Steps:
[ ] 1C.1  Copy codex-rs/plugin/ from reference
[ ] 1C.2  Rename: codex-plugin → orbit-code-plugin, codex_plugin → orbit_code_plugin
[ ] 1C.3  Rename dependency: codex-utils-plugins → orbit-code-utils-plugins
[ ] 1C.4  Copy codex-rs/utils/plugins/ from reference, rename same way
[ ] 1C.5  Add both to workspace
[ ] 1C.6  cargo check -p orbit-code-plugin
[ ] 1C.7  Repeat for: features, rollout, core-skills, instructions
```

**Checkpoint after Phase 1:**
```
cargo check --workspace 2>&1 | tail -5
```
Everything that compiled before still compiles. New crates compile too. Commit.

---

## Phase 2: Upstream-Only Modified Files (245 files, no conflicts)

**Goal:** Take upstream improvements to files we never touched.

**Risk:** LOW — but must apply rename transformation.

### Strategy

For each file, the workflow is:
1. Copy from `reference/codex-upstream/`
2. Apply rename sed
3. Verify the crate still compiles

### 2A — Foundation layer (protocol, config, hooks)

```
Steps:
[ ] 2A.1  List upstream-only modified files in protocol/:
         comm -12 /tmp/upstream_only.txt /tmp/upstream_modified.txt | grep codex-rs/protocol/
[ ] 2A.2  For each file: copy from reference, apply renames
[ ] 2A.3  cargo check -p orbit-code-protocol
[ ] 2A.4  cargo test -p orbit-code-protocol
[ ] 2A.5  Repeat for: config/, hooks/
```

### 2B — Utility crates

```
Steps:
[ ] 2B.1  Take upstream-only modifications in utils/*, rmcp-client/, network-proxy/,
         shell-command/, apply-patch/, shell-escalation/
[ ] 2B.2  Apply renames to each
[ ] 2B.3  cargo check for each changed crate
```

### 2C — Core upstream-only files (41 modified + 27 new)

```
Steps:
[ ] 2C.1  List upstream-only files in core/ that are MODIFICATIONS (not new):
         comm -12 /tmp/upstream_only.txt /tmp/upstream_modified.txt | grep codex-rs/core/
[ ] 2C.2  VERIFY each file doesn't exist in our Protected Code list above
[ ] 2C.3  Copy, rename, check
[ ] 2C.4  For NEW files upstream added to core/: copy, rename
[ ] 2C.5  cargo check -p orbit-code-core (may need to update mod.rs to add new modules)
```

### 2D — TUI upstream-only files (82 modified + 23 new)

```
Steps:
[ ] 2D.1  Take upstream-only modified TUI files
[ ] 2D.2  Take new TUI files (new features like plugin snapshots)
[ ] 2D.3  Apply renames
[ ] 2D.4  cargo check -p orbit-code-tui
```

### 2E — App-server layer

```
Steps:
[ ] 2E.1  Take upstream-only files in app-server/, app-server-protocol/
[ ] 2E.2  Apply renames
[ ] 2E.3  Take new v2 schema JSON files
[ ] 2E.4  cargo check -p orbit-code-app-server-protocol
[ ] 2E.5  cargo check -p orbit-code-app-server
```

**Checkpoint after Phase 2:**
```
cargo check --workspace 2>&1 | tail -5
cargo test -p orbit-code-protocol
cargo test -p orbit-code-hooks
```
Commit. Tag: `upstream-sync-phase2`.

---

## Phase 3: Easy Conflict Files (339 files, our changes ≤ 20 lines)

**Goal:** Merge files where we only made tiny changes (mostly renames).

**Risk:** MEDIUM — but our changes are small enough to re-apply manually.

### Strategy

For each file:
1. Take upstream's version from `reference/codex-upstream/`
2. Apply the rename transformation
3. Diff against our current version to see what our ~20 lines changed
4. Re-apply our small changes on top
5. Compile check

### 3A — Foundation easy conflicts

```
Steps:
[ ] 3A.1  For each easy conflict file in protocol/:
         - Copy upstream version
         - Apply renames
         - git diff to see our tiny changes
         - Re-apply our changes (usually just renames already applied by sed)
         - If our change was MORE than just renames, manually merge
[ ] 3A.2  cargo check -p orbit-code-protocol
[ ] 3A.3  cargo test -p orbit-code-protocol
[ ] 3A.4  Repeat for: config/, hooks/, execpolicy/
```

### 3B — Core easy conflicts

```
Steps:
[ ] 3B.1  List the easy conflict files in core/ (≤20 lines of our changes)
[ ] 3B.2  For each: take upstream, rename, re-apply our small tweaks
[ ] 3B.3  CRITICAL: Skip any file in the Protected Code list — those go to Phase 5
[ ] 3B.4  cargo check -p orbit-code-core after each batch of ~10 files
```

### 3C — TUI easy conflicts

```
Steps:
[ ] 3C.1  Take upstream TUI easy conflicts into tui/
[ ] 3C.2  For tui_app_server/ easy conflicts: check if the change also needs to go to our tui_app_server
[ ] 3C.3  cargo check -p orbit-code-tui
[ ] 3C.4  cargo check -p orbit-code-tui-app-server
```

### 3D — App-server easy conflicts

```
Steps:
[ ] 3D.1  Take upstream app-server, app-server-protocol easy conflicts
[ ] 3D.2  Apply renames, re-apply our small changes
[ ] 3D.3  cargo check -p orbit-code-app-server-protocol
[ ] 3D.4  cargo check -p orbit-code-app-server
```

### 3E — Everything else (CI, scripts, misc)

```
Steps:
[ ] 3E.1  .github/workflows/ — take upstream, re-apply our org/repo renames
[ ] 3E.2  Other misc files
```

**Checkpoint after Phase 3:**
```
cargo check --workspace 2>&1 | tail -5
cargo test -p orbit-code-protocol
cargo test -p orbit-code-core --lib -- --test-threads=1 2>&1 | tail -5
```
Commit. Tag: `upstream-sync-phase3`.

---

## Phase 4: Medium Conflict Files (262 files, our changes 21-100 lines)

**Goal:** Merge files where we made moderate changes.

**Risk:** HIGH — need to read both versions and blend.

### Strategy

For each file:
1. Generate a 3-way diff: fork base, our version, upstream version
2. Identify which hunks are ours vs theirs
3. Start from upstream's version (with renames)
4. Add back our hunks that upstream doesn't have
5. Compile check after each file

### 4A — Protocol + Config medium conflicts

```
Steps:
[ ] 4A.1  For each medium conflict in protocol/:
         git diff 01df50cf4..HEAD -- <file>          # Our changes
         git diff 01df50cf4..upstream/main -- <file>  # Their changes
[ ] 4A.2  Start from upstream version, apply renames, add our hunks back
[ ] 4A.3  cargo check + test
```

### 4B — Core medium conflicts (most files here)

```
Steps:
[ ] 4B.1  Work through core/ medium conflicts, SKIPPING Protected Code files
[ ] 4B.2  For config/*.rs files: preserve our permission mode additions
[ ] 4B.3  For exec*.rs files: preserve our changes
[ ] 4B.4  For each batch: cargo check -p orbit-code-core
```

### 4C — TUI medium conflicts

```
Steps:
[ ] 4C.1  Work through tui/ medium conflicts
[ ] 4C.2  Preserve our thinking token rendering code
[ ] 4C.3  Preserve our auth flow UI
[ ] 4C.4  cargo check -p orbit-code-tui
```

### 4D — App-server medium conflicts

```
Steps:
[ ] 4D.1  Work through app-server/ medium conflicts
[ ] 4D.2  Preserve our orbit_code_message_processor additions
[ ] 4D.3  cargo check + test
```

**Checkpoint after Phase 4:**
```
cargo check --workspace
cargo test -p orbit-code-protocol
cargo test -p orbit-code-core --lib -- --test-threads=1
cargo test -p orbit-code-tui --lib
```
Commit. Tag: `upstream-sync-phase4`.

---

## Phase 5: Hard Conflict Files (83 files, our changes > 100 lines)

**Goal:** Surgically merge the most heavily modified files.

**Risk:** EXTREME — this is where our core features live alongside upstream's biggest refactors.

### Strategy

Work file by file. For each:
1. Open both versions side-by-side (our version + reference/codex-upstream/ version)
2. Identify upstream additions (new functions, new match arms, new fields)
3. Add those INTO our version — don't replace our version with theirs
4. Test after every single file

### 5A — Cargo.toml files (workspace + per-crate)

```
Steps:
[ ] 5A.1  codex-rs/Cargo.toml — add new workspace members + deps for new crates
         DO NOT replace our orbit-code-* dependency names
         ADD new entries: orbit-code-plugin, orbit-code-features, orbit-code-rollout, etc.
[ ] 5A.2  Per-crate Cargo.toml files — add new dependencies upstream added
         Keep our package names, add their new deps (renamed)
[ ] 5A.3  cargo check --workspace
```

### 5B — Protocol (codex-rs/protocol/)

```
Steps:
[ ] 5B.1  protocol/src/protocol.rs — add new Op/EventMsg variants upstream added
         KEEP our existing variants intact
         ADD: new multi-agent v2 ops, spawn v2, etc.
[ ] 5B.2  protocol/src/models.rs — add new model types
[ ] 5B.3  protocol/src/permissions.rs — add new permission types
[ ] 5B.4  cargo test -p orbit-code-protocol
```

### 5C — Core agent loop (THE critical merge)

```
Steps:
[ ] 5C.1  core/src/codex.rs — THE most important file
         Start from OUR version (preserve Claude provider, thinking tokens)
         Read upstream's version for new additions:
           - Multi-agent v2 communication
           - Fork snapshot modes
           - Rollback context
           - Plugin integration hooks
         ADD new functions/match arms INTO our version
         DO NOT replace our handle_anthropic_*, thinking_token_*, provider routing code
[ ] 5C.2  cargo check -p orbit-code-core
[ ] 5C.3  core/src/client.rs — same approach
         Our multi-provider routing stays
         Add upstream's connector extraction changes
[ ] 5C.4  core/src/config/mod.rs — add upstream's new config fields
         Keep our permission mode fields
[ ] 5C.5  core/src/error.rs — add new error variants
[ ] 5C.6  core/src/exec.rs — add upstream exec changes
         Keep our modifications
[ ] 5C.7  cargo test -p orbit-code-core --lib -- --test-threads=1
```

### 5D — Core supporting files

```
Steps:
[ ] 5D.1  core/src/config/config_tests.rs (ours:897, theirs:447)
[ ] 5D.2  core/src/connectors.rs (ours:219, theirs:17) — keep our connector code
[ ] 5D.3  core/src/exec_policy*.rs — add upstream policy changes
[ ] 5D.4  core/src/guardian/* — add follow-up reminders, denial rationale
[ ] 5D.5  core/src/plugins/manager_tests.rs (ours:113, theirs:571) — add their test coverage
[ ] 5D.6  core/src/mcp_connection_manager.rs — add upstream improvements
[ ] 5D.7  core tests — add upstream test additions, keep our test files
[ ] 5D.8  cargo test -p orbit-code-core --lib -- --test-threads=1
```

### 5E — TUI files (the second hardest merge)

```
Steps:
[ ] 5E.1  tui/src/app.rs (ours:617, theirs:6151) — add upstream features into our version
         Keep our thinking token handling, auth flow
         Add: /title, /compact, plugin menu, collaboration mode refresh
[ ] 5E.2  tui/src/chatwidget.rs (ours:903, theirs:3688) — same approach
         Keep our thinking block rendering
         Add: plugin popups, new snapshot tests
[ ] 5E.3  tui/src/lib.rs (ours:239, theirs:1239)
[ ] 5E.4  tui/src/streaming/controller.rs — keep our thinking token streaming
[ ] 5E.5  tui/src/bottom_pane/*.rs — add upstream improvements
[ ] 5E.6  tui/src/chatwidget/tests.rs — add upstream test cases
[ ] 5E.7  cargo check -p orbit-code-tui
[ ] 5E.8  cargo test -p orbit-code-tui --lib
```

### 5F — tui_app_server (mirror critical changes)

```
Steps:
[ ] 5F.1  Since upstream merged tui_app_server → tui, their tui_app_server is gone
[ ] 5F.2  We keep ours but port relevant upstream changes from their merged tui/
[ ] 5F.3  tui_app_server/src/app.rs — add upstream features
[ ] 5F.4  tui_app_server/src/chatwidget.rs — add upstream features
[ ] 5F.5  cargo check -p orbit-code-tui-app-server
```

### 5G — App-server layer

```
Steps:
[ ] 5G.1  app-server-protocol/src/protocol/v2.rs (ours:333, theirs:548) — add new types
[ ] 5G.2  app-server/src/message_processor.rs — add upstream improvements
[ ] 5G.3  app-server/src/config_api.rs — add upstream changes
[ ] 5G.4  app-server-client/src/lib.rs — add upstream changes
[ ] 5G.5  cargo test -p orbit-code-app-server-protocol
[ ] 5G.6  cargo test -p orbit-code-app-server --lib
```

### 5H — Exec and CLI

```
Steps:
[ ] 5H.1  exec/src/lib.rs (ours:251, theirs:984)
[ ] 5H.2  cli/src/main.rs (ours:359, theirs:589)
[ ] 5H.3  state/* files
[ ] 5H.4  cargo check for each
```

### 5I — Cargo.lock (dead last)

```
Steps:
[ ] 5I.1  Delete Cargo.lock
[ ] 5I.2  cargo generate-lockfile
[ ] 5I.3  cargo check --workspace
```

**Checkpoint after Phase 5:**
```
cargo check --workspace
cargo test -p orbit-code-protocol
cargo test -p orbit-code-core --lib -- --test-threads=1
cargo test -p orbit-code-tui --lib
cargo test -p orbit-code-app-server-protocol
cargo test -p orbit-code-app-server --lib
```
Commit. Tag: `upstream-sync-phase5`.

---

## Phase 6: Verification & Cleanup

```
Steps:
[ ] 6.1  cargo check --workspace — full workspace compiles
[ ] 6.2  cargo test -p orbit-code-protocol
[ ] 6.3  cargo test -p orbit-code-core --lib -- --test-threads=1
[ ] 6.4  cargo test -p orbit-code-tui --lib
[ ] 6.5  cargo test -p orbit-code-app-server --lib
[ ] 6.6  just fmt — format everything
[ ] 6.7  just fix -p orbit-code-core — clippy
[ ] 6.8  just fix -p orbit-code-tui — clippy
[ ] 6.9  Verify protected code still exists:
         grep -r "anthropic_bridge" codex-rs/core/src/  # Our bridge exists
         grep -r "thinking" codex-rs/tui/src/           # Thinking tokens exist
         grep -r "ORBIT_HOME" codex-rs/utils/           # Our env var exists
         ls codex-rs/anthropic/src/                      # Our crate exists
[ ] 6.10 Update docs/upstream/upstream.md with new sync status
[ ] 6.11 Update docs/tracked/orbit-code-roadmap.md
```

---

## Abort & Rollback

At ANY point if things go sideways:

```bash
# See what changed
git diff --stat

# Undo everything back to last commit
git checkout -- .
git clean -fd

# Nuclear option: back to main
git checkout main
git branch -D upstream-sync-phase1
```

---

## File Count Summary

| Phase | Files | Risk | Estimated Sessions |
|-------|-------|------|-------------------|
| Phase 0 | 0 | None | 1 (setup) |
| Phase 1 | ~50 | Low | 1-2 (new crates) |
| Phase 2 | ~245 | Low | 2-3 (copy + rename) |
| Phase 3 | ~339 | Medium | 3-5 (re-apply small changes) |
| Phase 4 | ~262 | High | 5-8 (3-way merge) |
| Phase 5 | ~83 | Extreme | 8-15 (surgical line-by-line) |
| Phase 6 | 0 | None | 1 (verify) |
| **Total** | **~979** | | **~20-35 sessions** |

---

## Decision Log

Track decisions made during sync here:

| Date | Decision | Rationale |
|------|----------|-----------|
| | | |
