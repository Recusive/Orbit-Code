# Rename codex → orbit-code Implementation Plan (v5 — Final)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebrand internal naming from "codex" to "orbit-code" at the application level. Provider/API endpoints, wire protocol identifiers, and sandbox contracts are untouched.

**Architecture:** Broad sed rename of internal naming (crate names, imports, strings, config paths), then targeted restore of ~15 preserved identifiers. Dual-read shims for backwards compatibility with existing user data.

**Tech Stack:** Rust (Cargo workspace), GNU sed, npm (pnpm), Python.

---

## Scope

| What | Old | New |
|------|-----|-----|
| Brand name | Codex | Orbit Code |
| Binary name | `codex` | `orbit-code` |
| Config directory | `~/.codex/` | `~/.orbit/` |
| Project config | `.codex/` | `.orbit/` |
| System config | `/etc/codex/` | `/etc/orbit/` |
| Home env var | `CODEX_HOME` | `ORBIT_HOME` |
| Internal env vars | `CODEX_TUI_*`, `CODEX_CI`, etc. | `ORBIT_TUI_*`, `ORBIT_CI`, etc. |
| Cargo crate names | `codex-core` | `orbit-code-core` |
| Rust imports | `codex_core::` | `orbit_code_core::` |
| npm packages | `@openai/codex` | `@orbit.build/orbit-code` |
| Python package | `codex-app-server-sdk` | `orbit-code-app-server-sdk` |
| User-Agent originator | `codex_cli_rs` | `orbit_code_cli_rs` |
| BUILD.bazel targets | `codex_*` | `orbit_code_*` |
| Telemetry events | `codex.*` | `orbit_code.*` |
| Keyring service | `"Codex Auth"` | `"Orbit Code Auth"` (dual-read) |

## Explicitly Preserved (DO NOT RENAME)

| Identifier | Why |
|------------|-----|
| `"codex/sandbox-state"`, `"codex/sandbox-state/update"` | MCP wire protocol |
| `"codex"` MCP tool name, `"codex-reply"` | MCP client contract |
| `CODEX_SANDBOX_ENV_VAR`, `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` | AGENTS.md forbids |
| `"CODEX_SANDBOX"`, `"CODEX_SANDBOX_NETWORK_DISABLED"` env values | Sandbox runtime sets these |
| `"codex"` realtime API tool name | OpenAI wire protocol |
| `"x-openai-internal-codex-residency"` header | OpenAI internal routing |
| `"OpenAI-Beta"` header | OpenAI API requirement |
| `OPENAI_API_KEY`, `OPENAI_BASE_URL` env vars | Provider standard |
| `CODEX_API_KEY` env var VALUE | External contract (dual-read with `ORBIT_API_KEY`) |
| All `openai.com` / `api.openai.com` URLs in source | Provider endpoints — we are application level, not provider level |
| `gpt-*`, `*-codex` model name strings | Model identifiers |
| App-server RPC methods (`thread/start`, etc.) | Already generic, no "codex" in them |
| `codex-rs/` directory name | Upstream sync compatibility |
| `NOTICE` file contents | Legal attribution |

---

## File Structure

**Files created:**
- `scripts/rename-codex-to-orbit-code.sh`

**Files renamed on disk:**
- `sdk/python/src/codex_app_server/` → `sdk/python/src/orbit_code_app_server/`
- `sdk/python-runtime/src/codex_cli_bin/` → `sdk/python-runtime/src/orbit_code_cli_bin/`

**Files modified:**
- 77 `Cargo.toml` files, ~800 `.rs` files, 76+ `BUILD.bazel` files
- `codex-cli/package.json`, `codex-cli/bin/codex.js`
- `sdk/python/**`, `sdk/typescript/**`, `shell-tool-mcp/package.json`
- `package.json`, `justfile`, `defs.bzl`, `.config/nextest.toml`
- All `CLAUDE.md` and `AGENTS.md` inside `codex-rs/`
- `docs/**`

---

### Task 1: Build the Rename Script

**Files:**
- Create: `scripts/rename-codex-to-orbit-code.sh`

- [ ] **Step 1: Create the rename script**

Requires GNU sed (`brew install gnu-sed` on macOS). All `find` calls use `-exec $S -i ... {} +` — no shell functions in subshells.

```bash
#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

if command -v gsed &>/dev/null; then S=gsed
elif sed --version 2>/dev/null | grep -q GNU; then S=sed
else echo "ERROR: GNU sed required. On macOS: brew install gnu-sed"; exit 1; fi

# ==========================================================
# PHASE 1: Cargo.toml — crate names, deps, aliases
# ==========================================================
echo "=== Phase 1: Cargo.toml ==="
find codex-rs -name 'Cargo.toml' -exec $S -i \
  -e 's/codex-/orbit-code-/g' \
  -e 's/codex_/orbit_code_/g' {} +
$S -i 's/name = "orbit-code-cli"/name = "orbit-code"/' codex-rs/cli/Cargo.toml

# ==========================================================
# PHASE 2: .rs — imports and module names
# ==========================================================
echo "=== Phase 2: Rust imports ==="
find codex-rs -name '*.rs' -exec $S -i 's/codex_/orbit_code_/g' {} +

# ==========================================================
# PHASE 3: .rs — CODEX_ env vars → ORBIT_ (broad)
# ==========================================================
echo "=== Phase 3: Env vars ==="
find codex-rs -name '*.rs' -exec $S -i 's/CODEX_/ORBIT_/g' {} +

# ==========================================================
# PHASE 4: .rs — user-facing strings
# ==========================================================
echo "=== Phase 4: User strings ==="
find codex-rs -name '*.rs' -exec $S -i \
  -e 's/"Codex"/"Orbit Code"/g' \
  -e 's/"Codex (Dev)"/"Orbit Code (Dev)"/g' \
  -e 's/"Codex (Agent)"/"Orbit Code (Agent)"/g' \
  -e 's/"Codex (Nightly)"/"Orbit Code (Nightly)"/g' \
  -e 's/"Codex (Alpha)"/"Orbit Code (Alpha)"/g' \
  -e 's/"Codex (Beta)"/"Orbit Code (Beta)"/g' \
  -e 's/Codex MCP Credentials/Orbit Code MCP Credentials/g' \
  -e 's/Codex Auth/Orbit Code Auth/g' \
  -e 's/Codex blocked/Orbit Code blocked/g' \
  -e 's/Codex can /Orbit Code can /g' \
  -e 's/Codex runtime/Orbit Code runtime/g' \
  -e 's/Codex network/Orbit Code network/g' {} +

# ==========================================================
# PHASE 5: .rs — config paths (use .orbit not .orbit-code)
# ==========================================================
echo "=== Phase 5: Config paths ==="
find codex-rs -name '*.rs' -exec $S -i \
  -e 's|/etc/codex/|/etc/orbit/|g' \
  -e 's|\.codex/|.orbit/|g' \
  -e 's|"codex-arg0"|"orbit-code-arg0"|g' \
  -e 's/--codex-run-as-apply-patch/--orbit-code-run-as-apply-patch/g' {} +

# ==========================================================
# RESTORE: Preserved identifiers
# ==========================================================
echo "=== Restoring preserved identifiers ==="

# Sandbox const names (AGENTS.md rule)
find codex-rs -name '*.rs' -exec $S -i \
  -e 's/CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR/CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR/g' \
  -e 's/CODEX_SANDBOX_ENV_VAR/CODEX_SANDBOX_ENV_VAR/g' {} +

# Sandbox env var VALUES (string literals)
find codex-rs -name '*.rs' -exec $S -i \
  -e 's/"CODEX_SANDBOX_NETWORK_DISABLED"/"CODEX_SANDBOX_NETWORK_DISABLED"/g' \
  -e 's/"CODEX_SANDBOX"/"CODEX_SANDBOX"/g' {} +

# CODEX_API_KEY env var value (external contract, dual-read added in Task 3)
find codex-rs -name '*.rs' -exec $S -i \
  's/"ORBIT_API_KEY"/"CODEX_API_KEY"/g' {} +

# MCP sandbox-state wire protocol
$S -i \
  -e 's/"orbit-code\/sandbox-state"/"codex\/sandbox-state"/g' \
  -e 's/"orbit-code\/sandbox-state\/update"/"codex\/sandbox-state\/update"/g' \
  codex-rs/core/src/mcp_connection_manager.rs

# MCP tool names
$S -i \
  -e 's/"orbit-code"/"codex"/g' \
  -e 's/"orbit-code-reply"/"codex-reply"/g' \
  codex-rs/mcp-server/src/message_processor.rs

# Realtime API tool name
find codex-rs/codex-api -name '*.rs' -path '*/realtime_websocket/*' \
  -exec $S -i 's/"orbit-code"/"codex"/g' {} +

# Internal OpenAI header
$S -i 's/"x-openai-internal-orbit-code-residency"/"x-openai-internal-codex-residency"/g' \
  codex-rs/core/src/default_client.rs 2>/dev/null || true

# Restore any openai.com URLs that got mangled by codex_ rename
# (we do NOT rewrite provider endpoints)
find codex-rs -name '*.rs' -exec $S -i \
  -e 's/orbit_code_\.com/openai.com/g' {} + 2>/dev/null || true

# ==========================================================
# NON-RUST FILES
# ==========================================================
echo "=== BUILD.bazel ==="
find codex-rs -name 'BUILD.bazel' -exec $S -i \
  -e 's/codex_/orbit_code_/g' \
  -e 's/codex-/orbit-code-/g' \
  -e 's/"codex"/"orbit-code"/g' {} +
$S -i -e 's/codex_/orbit_code_/g' -e 's/codex-/orbit-code-/g' -e 's/"codex"/"orbit-code"/g' \
  defs.bzl 2>/dev/null || true

echo "=== nextest.toml ==="
$S -i 's/codex-/orbit-code-/g' codex-rs/.config/nextest.toml 2>/dev/null || true

echo "=== npm packages ==="
for f in codex-cli/package.json shell-tool-mcp/package.json sdk/typescript/package.json; do
  [ -f "$f" ] && $S -i \
    -e 's/@openai\/codex/@orbit.build\/orbit-code/g' \
    -e 's/"codex"/"orbit-code"/g' \
    -e 's/codex-/orbit-code-/g' \
    -e 's/CODEX_/ORBIT_/g' "$f"
done
[ -f codex-cli/bin/codex.js ] && $S -i \
  -e 's/@openai\/codex/@orbit.build\/orbit-code/g' \
  -e 's/codex-/orbit-code-/g' \
  -e 's/CODEX_/ORBIT_/g' codex-cli/bin/codex.js
[ -f codex-rs/responses-api-proxy/npm/package.json ] && $S -i \
  -e 's/@openai\/codex/@orbit.build\/orbit-code/g' \
  -e 's/codex-/orbit-code-/g' \
  codex-rs/responses-api-proxy/npm/package.json
$S -i -e 's/codex-monorepo/orbit-code-monorepo/g' -e 's/codex-/orbit-code-/g' package.json

echo "=== Build scripts ==="
find codex-cli/scripts -name '*.py' -exec $S -i \
  -e 's/codex/orbit-code/g' -e 's/Codex/Orbit Code/g' -e 's/@openai/@orbit.build/g' {} + 2>/dev/null || true
[ -f scripts/stage_npm_packages.py ] && $S -i \
  -e 's/codex/orbit-code/g' -e 's/@openai/@orbit.build/g' \
  scripts/stage_npm_packages.py 2>/dev/null || true

echo "=== Python SDK ==="
[ -f sdk/python/pyproject.toml ] && $S -i \
  -e 's/codex-app-server/orbit-code-app-server/g' \
  -e 's/codex_app_server/orbit_code_app_server/g' \
  -e 's/"codex"/"orbit-code"/g' \
  -e 's/Codex/Orbit Code/g' \
  -e 's|openai/codex|Recursive/Orbit-Code|g' \
  sdk/python/pyproject.toml
[ -d sdk/python/src/codex_app_server ] && mv sdk/python/src/codex_app_server sdk/python/src/orbit_code_app_server
find sdk/python -name '*.py' -exec $S -i \
  -e 's/codex_app_server/orbit_code_app_server/g' \
  -e 's/codex-app-server/orbit-code-app-server/g' {} + 2>/dev/null || true
if [ -d sdk/python-runtime ]; then
  [ -f sdk/python-runtime/pyproject.toml ] && $S -i \
    -e 's/codex-cli-bin/orbit-code-cli-bin/g' \
    -e 's/codex_cli_bin/orbit_code_cli_bin/g' \
    -e 's|openai/codex|Recursive/Orbit-Code|g' \
    sdk/python-runtime/pyproject.toml
  [ -d sdk/python-runtime/src/codex_cli_bin ] && mv sdk/python-runtime/src/codex_cli_bin sdk/python-runtime/src/orbit_code_cli_bin
  find sdk/python-runtime -name '*.py' -exec $S -i 's/codex_cli_bin/orbit_code_cli_bin/g' {} + 2>/dev/null || true
fi
find sdk/typescript -name '*.ts' -exec $S -i \
  -e 's/codex_/orbit_code_/g' -e 's/codex-/orbit-code-/g' -e 's/Codex/OrbitCode/g' {} + 2>/dev/null || true

echo "=== Prompt templates ==="
find codex-rs \( -name '*.md' -path '*/templates/*' -o -name '*.txt' -path '*/prompts/*' \) \
  -exec $S -i -e 's/Codex/Orbit Code/g' -e 's/codex/orbit-code/g' {} +

echo "=== Justfile, scripts, docs ==="
[ -f justfile ] && $S -i -e 's/codex-/orbit-code-/g' -e 's/codex /orbit-code /g' justfile
find scripts -name '*.sh' -not -name 'rename-*' \
  -exec $S -i -e 's/codex/orbit-code/g' -e 's/Codex/Orbit Code/g' {} + 2>/dev/null || true
find docs -name '*.md' -exec $S -i \
  -e 's/Codex CLI/Orbit Code/g' -e 's/Codex/Orbit Code/g' \
  -e 's/@openai\/codex/@orbit.build\/orbit-code/g' \
  -e 's|openai/codex|Recursive/Orbit-Code|g' \
  -e 's/codex-/orbit-code-/g' {} + 2>/dev/null || true

echo "=== CLAUDE.md and AGENTS.md ==="
find codex-rs \( -name 'CLAUDE.md' -o -name 'AGENTS.md' \) -exec $S -i \
  -e 's/codex-/orbit-code-/g' -e 's/codex_/orbit_code_/g' \
  -e 's/Codex/Orbit Code/g' -e 's/CODEX_/ORBIT_/g' {} +

echo "=== Snapshot cleanup (stale codex_ prefixed only) ==="
find codex-rs -name '*.snap' -path '*/snapshots/*' | while read -r f; do
  basename "$f" | grep -q '^codex_' && rm "$f"
done
find codex-rs -name '*.snap.new' -delete 2>/dev/null || true

echo "=== Regenerate Cargo.lock ==="
(cd codex-rs && rm -f Cargo.lock && cargo generate-lockfile 2>/dev/null) || echo "WARNING: Cargo.lock deferred"

echo "=== Bazel lock ==="
just bazel-lock-update 2>/dev/null || echo "WARNING: Bazel lock skipped"

echo "=== RENAME COMPLETE === Next: cd codex-rs && cargo check"
```

- [ ] **Step 2: Make executable and commit**

```bash
chmod +x scripts/rename-codex-to-orbit-code.sh
git add scripts/rename-codex-to-orbit-code.sh
git commit -m "Add rename script: codex → orbit-code"
git push origin main
```

---

### Task 2: Run the Rename Script

- [ ] **Step 1: Backup branch**

Run: `git checkout -b pre-rename-backup && git checkout main`

- [ ] **Step 2: Install GNU sed (macOS)**

Run: `brew install gnu-sed`

- [ ] **Step 3: Run**

Run: `./scripts/rename-codex-to-orbit-code.sh 2>&1 | tee /tmp/rename.log`

- [ ] **Step 4: Verify crate names renamed**

Run: `grep -r 'name = "codex' codex-rs/*/Cargo.toml codex-rs/Cargo.toml 2>/dev/null | grep -v '#'`
Expected: No output

- [ ] **Step 5: Verify sandbox consts PRESERVED**

Run: `grep -n 'CODEX_SANDBOX' codex-rs/core/src/spawn.rs`
Expected: `CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR` and `CODEX_SANDBOX_ENV_VAR` present

- [ ] **Step 6: Verify MCP wire protocol PRESERVED**

Run: `grep -n 'codex/sandbox-state' codex-rs/core/src/mcp_connection_manager.rs`
Expected: Both strings present

Run: `grep -n '"codex"' codex-rs/mcp-server/src/message_processor.rs`
Expected: MCP tool name present

- [ ] **Step 7: Verify OpenAI URLs NOT rewritten**

Run: `grep -rn 'openai\.com' codex-rs/ --include='*.rs' | head -5`
Expected: `api.openai.com` URLs still present

- [ ] **Step 8: Verify .orbit (not .orbit-code)**

Run: `grep -rn '\.orbit-code' codex-rs/ --include='*.rs' | head -3`
Expected: No output

Run: `grep -rn '\.orbit/' codex-rs/ --include='*.rs' | head -3`
Expected: References to `.orbit/` config paths

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "Rename codex → orbit-code across codebase (preserves wire contracts)"
```

---

### Task 3: Add Dual-Read Compatibility Shims

No destructive migration. Code reads both old and new names; new takes precedence.

**Files:**
- Modify: `codex-rs/utils/home-dir/src/lib.rs`
- Modify: `codex-rs/core/src/auth/storage.rs:135`
- Modify: `codex-rs/core/src/auth.rs:378`
- Modify: `codex-rs/core/src/config_loader/mod.rs`

- [ ] **Step 1: Home directory dual-read**

In the home dir function (now `find_orbit_home()`):

```rust
pub fn find_orbit_home() -> io::Result<PathBuf> {
    // 1. ORBIT_HOME env var (new)
    if let Ok(val) = env::var("ORBIT_HOME") { ... }
    // 2. CODEX_HOME env var (legacy)
    if let Ok(val) = env::var("CODEX_HOME") { ... }
    // 3. ~/.orbit (new default)
    let new_home = dirs::home_dir()?.join(".orbit");
    if new_home.exists() { return Ok(new_home); }
    // 4. ~/.codex (legacy fallback)
    let legacy = dirs::home_dir()?.join(".codex");
    if legacy.exists() { return Ok(legacy); }
    // 5. Fresh install — new path
    Ok(new_home)
}
```

- [ ] **Step 2: Keyring dual-read (`core/src/auth/storage.rs:135`)**

```rust
const KEYRING_SERVICE: &str = "Orbit Code Auth";
const LEGACY_KEYRING_SERVICE: &str = "Codex Auth";

// load(): try new service, fall back to legacy
// save(): always write to new service
```

- [ ] **Step 3: API key dual-read (`core/src/auth.rs`)**

```rust
pub const ORBIT_API_KEY_ENV_VAR: &str = "ORBIT_API_KEY";
pub const LEGACY_API_KEY_ENV_VAR: &str = "CODEX_API_KEY";

fn load_api_key_from_env() -> Option<String> {
    env::var(ORBIT_API_KEY_ENV_VAR).ok()
        .or_else(|| env::var(LEGACY_API_KEY_ENV_VAR).ok())
}
```

Note: `OPENAI_API_KEY` read separately (provider standard, unchanged).

- [ ] **Step 4: Project config dual-read (`core/src/config_loader/mod.rs`)**

Check both `.orbit/config.toml` and `.codex/config.toml` per directory. New takes precedence. Same for `.orbit/skills/` and `.codex/skills/`.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "Add dual-read compat shims for legacy .codex paths, env vars, and keyring"
git push origin main
```

---

### Task 4: Fix Compilation Errors

- [ ] **Step 1: cargo check**

Run: `cd codex-rs && cargo check 2>&1 | head -100`

- [ ] **Step 2: Fix errors iteratively**

Watch for:
- `codec` accidentally renamed (not `codex`)
- `[patch.crates-io]` URLs mangled
- External crates with "codex" in path that aren't ours
- String literals that should keep "codex" (wire protocol — verify restores worked)

- [ ] **Step 3: Iterate until clean**

Run: `cd codex-rs && cargo check`
Expected: `Finished`

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "Fix compilation errors from rename"
git push origin main
```

---

### Task 5: Regenerate Schemas, Snapshots, and Fix Tests

- [ ] **Step 1: Regenerate schemas**

```bash
just write-app-server-schema
just write-config-schema
just write-hooks-schema
```

- [ ] **Step 2: Update MODULE.bazel.lock**

Run: `just bazel-lock-update && just bazel-lock-check`

- [ ] **Step 3: Run core tests**

Run: `cd codex-rs && cargo test -p orbit-code-core 2>&1 | tail -30`

- [ ] **Step 4: Run TUI tests + accept new snapshots**

```bash
cd codex-rs
cargo test -p orbit-code-tui 2>&1 | tail -30
cargo insta accept -p orbit-code-tui
cargo test -p orbit-code-tui-app-server 2>&1 | tail -30
cargo insta accept -p orbit-code-tui-app-server
```

- [ ] **Step 5: Run full test suite**

Run: `cd codex-rs && cargo nextest run --no-fail-fast 2>&1 | tail -50`

- [ ] **Step 6: Fix remaining failures and commit**

```bash
git add -A
git commit -m "Regenerate schemas/snapshots, fix tests after rename"
git push origin main
```

---

### Task 6: Build Binary and Smoke Test

- [ ] **Step 1: Build**

Run: `cd codex-rs && cargo build --bin orbit-code`

- [ ] **Step 2: Verify no "codex" in help (except preserved wire identifiers)**

Run: `cd codex-rs && ./target/debug/orbit-code --help 2>&1 | grep -i codex`
Expected: No output

- [ ] **Step 3: Verify "Orbit Code" branding**

Run: `cd codex-rs && ./target/debug/orbit-code --help | head -5`
Expected: "Orbit Code" in output

- [ ] **Step 4: Commit and push**

```bash
git add -A
git commit -m "orbit-code binary builds and runs"
git push origin main
```

---

### Task 7: Final Verification

- [ ] **Step 1: Grep for remaining "codex" in source**

```bash
grep -rn 'codex' --include='*.rs' --include='*.toml' --include='*.json' \
  --include='*.py' --include='*.ts' --include='*.js' --include='*.bazel' \
  codex-rs/ codex-cli/ sdk/ shell-tool-mcp/ \
  | grep -v node_modules | grep -v target/ | grep -v CLAUDE.md \
  | grep -v AGENTS.md | grep -v NOTICE | grep -v .lock | grep -v .snap \
  | head -50
```

Expected: Only preserved identifiers (sandbox consts/values, MCP tool names, wire protocol, OpenAI URLs, model names, `codex-rs/` directory path).

- [ ] **Step 2: Final build + test**

Run: `cd codex-rs && cargo build && cargo nextest run --no-fail-fast 2>&1 | tail -20`

- [ ] **Step 3: Push**

```bash
git push origin main
```

---

## Post-Rename: Upstream Sync

```bash
git fetch upstream
git checkout -b upstream-merge
git merge upstream/main                    # resolve conflicts
./scripts/rename-codex-to-orbit-code.sh    # re-apply rename
cd codex-rs && cargo check                 # fix edge cases
cargo nextest run                          # verify
git checkout main && git merge upstream-merge
git push origin main
```
