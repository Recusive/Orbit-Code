# Plan: Remove `.codex` Legacy Fallback — Go `.orbit` Only

## Context

Orbit Code is a fork of OpenAI's Codex CLI. The fork currently supports **both** `.codex` and `.orbit` as config directories — a backward-compatibility mechanism for migrating upstream Codex users. Since Orbit Code has **zero users** and is diverging from upstream (adding Claude model support, Anthropic-specific config), the dual fallback adds complexity with no benefit. Additionally, loading a `.codex/config.toml` containing Orbit-specific fields would either fail to parse or silently ignore them.

**Goal:** Remove all `.codex` directory references, `CODEX_HOME` env var support, and `CODEX_API_KEY` env var fallback. The only config directory is `.orbit`.

**Out of scope:** `.codex-plugin` (plugin metadata convention) — separate concern, separate rename.

---

## Phase 1: Foundation — Home Directory Resolution

**Crate:** `orbit-code-utils-home-dir`
**File:** `codex-rs/utils/home-dir/src/lib.rs`

- Remove constants `LEGACY_CODEX_HOME_ENV_VAR` and `LEGACY_CODEX_HOME_DIR_NAME` (lines 7, 9)
- Simplify `find_orbit_home()` (lines 23-35): only read `ORBIT_HOME` env var
- Remove `find_codex_home()` wrapper (lines 41-43)
- Simplify `find_orbit_home_from_envs()` (lines 45-70): remove `legacy_codex_home_env` parameter and `~/.codex` fallback. Logic becomes: check `orbit_home_env` → check `~/.orbit` exists → default `~/.orbit`
- Remove 3 legacy tests: `find_legacy_codex_home_env_missing_path_is_fatal`, `find_orbit_home_env_takes_precedence_over_legacy_codex_home`, `find_orbit_home_without_env_falls_back_to_existing_legacy_codex_dir`
- Update remaining test signatures (remove second parameter from `find_orbit_home_from_envs` calls)

**Verify:** `cargo test -p orbit-code-utils-home-dir`

---

## Phase 2: Auth — API Key and Keyring Legacy Removal

**Crate:** `orbit-code-core`

**`core/src/auth.rs`**
- Remove `LEGACY_CODEX_API_KEY_ENV_VAR` constant (line 219)
- Simplify `read_orbit_api_key_from_env()` (lines 228-239): remove `.or_else` fallback to `CODEX_API_KEY`

**`core/src/auth_tests.rs`**
- Remove 2 tests: `read_orbit_api_key_from_env_prefers_orbit_over_legacy_codex_env_var`, `read_orbit_api_key_from_env_falls_back_to_legacy_codex_env_var`

**`core/src/auth/storage.rs`**
- Remove `LEGACY_KEYRING_SERVICE` constant (line 435)
- Simplify `candidate_store_keys()` (lines 451-466): remove the `.codex` key generation branch. Return just `vec![compute_store_key(orbit_code_home)?]`
- Simplify `KeyringAuthStorage::load()` (lines 517-528): remove `LEGACY_KEYRING_SERVICE` loop. Only check `KEYRING_SERVICE`
- Simplify `KeyringAuthStorage::delete()` (lines 540-556): remove `LEGACY_KEYRING_SERVICE` from service iteration

**`core/src/auth/storage_tests.rs`**
- Remove `keyring_auth_storage_load_falls_back_to_legacy_service_and_default_codex_path`
- Update `keyring_auth_storage_compute_store_key_for_home_directory`: change `~/.codex` to `~/.orbit` and update expected hash

**Verify:** `cargo test -p orbit-code-core -- auth`

---

## Phase 3: Config Loader — Project Directory Scanning

**Crate:** `orbit-code-core`

**`core/src/config_loader/mod.rs`**
- Line 822: change `[".codex", ".orbit"]` to `[".orbit"]`
- Update doc comments (lines 98-103): remove `.codex` references in layer documentation

**`core/src/config_loader/tests.rs`**
- All 9 test functions: rename `.codex` → `.orbit` in fixture paths (~30 occurrences)
- Rename `dot_codex` variables → `dot_orbit`

**`core/src/config/mod.rs`**
- Remove `find_codex_home()` dead-code alias (line ~3024)

**`core/src/config/config_tests.rs`**
- Update 6+ test functions: `.codex` → `.orbit` in fixture paths (~8 occurrences)

**Verify:** `cargo test -p orbit-code-core -- config_loader` then `cargo test -p orbit-code-core -- config_tests`

---

## Phase 4: Skills Loader — Legacy Fallback Removal

**Crate:** `orbit-code-core`

**`core/src/skills/loader.rs`**
- Remove lines 274-281: the explicit `.codex/skills` fallback when config folder is `.orbit`

**`core/src/skills/loader_tests.rs`**
- Line 27: change `REPO_ROOT_CONFIG_DIR_NAME` from `".codex"` to `".orbit"`
- Update test fixture paths

**Verify:** `cargo test -p orbit-code-core -- skills`

---

## Phase 5: Protocol — Sandbox Protection Lists

**Crate:** `orbit-code-protocol`

**`protocol/src/protocol.rs`**
- Line 789: update comment `.codex` → `.orbit`
- Line 1040: change `[".agents", ".codex"]` → `[".agents", ".orbit"]`
- Line 1042: rename `top_level_codex` → `top_level_orbit`
- Update tests (~4 occurrences)

**`protocol/src/permissions.rs`**
- Line 1050-1052: same changes as protocol.rs
- Update all test code creating `.codex` dirs → `.orbit` (~21 occurrences)
- Rename `expected_codex` variables → `expected_orbit`

**Verify:** `cargo test -p orbit-code-protocol`

---

## Phase 6: Platform Sandboxes

**Linux** (`orbit-code-linux-sandbox`):
- `linux-sandbox/src/bwrap.rs`: update comments and path lists
- `linux-sandbox/tests/suite/landlock.rs`: update 6 test occurrences

**macOS** (`orbit-code-core`):
- `core/src/seatbelt_tests.rs`: update ~15 test occurrences

**Windows** (`orbit-code-windows-sandbox-rs`):
- `windows-sandbox-rs/src/workspace_acl.rs`, `setup_main_win.rs`, `allow.rs`, `cap.rs`, `elevated/cwd_junction.rs`, `helper_materialization.rs`: update path lists and comments

**Verify:** `cargo test -p orbit-code-core -- seatbelt` (macOS), `just fix -p orbit-code-linux-sandbox`, `just fix -p orbit-code-windows-sandbox-rs`

---

## Phase 7: App-Server Protocol

**Crate:** `orbit-code-app-server-protocol`

**`app-server-protocol/src/protocol/v2.rs`**
- Line 474: update comment `.codex/` → `.orbit/`
- Line 904: update comment `~/.codex` → `~/.orbit`
- Lines 7558, 7584: update test example paths

**Then regenerate schemas:**
```bash
just write-app-server-schema
just write-app-server-schema --experimental
```

**Verify:** `cargo test -p orbit-code-app-server-protocol`

---

## Phase 8: TUI (Mirrored)

**Both `tui/` and `tui_app_server/`:**
- `src/debug_config.rs`: 4 test path occurrences each
- `src/bottom_pane/list_selection_view.rs`: line 1172 each
- `src/model_migration.rs`: 5 occurrences each

**Verify:** `cargo test -p orbit-code-tui` and `cargo test -p orbit-code-tui-app-server`
Accept snapshots if needed: `cargo insta accept -p orbit-code-tui`

---

## Phase 9: Remaining Source Files (Bulk)

Smaller changes (~1-5 occurrences each):
- `core/src/external_agent_config.rs` and `external_agent_config_tests.rs`
- `core/src/analytics_client_tests.rs`
- `core/src/plugins/manager_tests.rs`
- `core/src/agent/control.rs` and `control_tests.rs`
- `core/src/rollout/recorder.rs`, `rollout/list.rs`
- `core/src/guardian/review_session.rs`
- `core/src/orbit_code_thread.rs`
- `core/src/message_history.rs`
- `core/src/file_watcher_tests.rs`, `project_doc_tests.rs`
- `config/src/state.rs` (line 97 comment)
- `config/src/config_requirements.rs`
- `arg0/src/lib.rs`
- `state/src/bin/logs_client.rs`
- `core/src/config/types.rs` (line 336)
- `core/src/config/service.rs` (line 413)
- `core/src/model_provider_info.rs` (line 5)
- `utils/cli/src/config_override.rs` (line 21)
- `core/src/tools/handlers/multi_agents_tests.rs`

---

## Phase 10: Integration Tests (Bulk)

~50 test files under `core/tests/suite/` reference `.codex` in `.join(".codex")` calls. Heaviest files:

| File | Occurrences |
|------|-------------|
| `realtime_conversation.rs` | ~91 |
| `compact_remote.rs` | ~73 |
| `collaboration_instructions.rs` | ~45 |
| `compact.rs` | ~36 |
| `model_switching.rs` | ~36 |
| `personality.rs` | ~30 |

Also: `app-server/tests/suite/v2/` files (config_rpc, thread_start, plugin_list, etc.)

This is mechanical: `.join(".codex")` → `.join(".orbit")` throughout.

**Verify:** `cargo test -p orbit-code-core` (full suite)

---

## Phase 11: Documentation

Update `.codex` → `.orbit` in ~53 markdown files:
- `codex-rs/README.md`
- `docs/codex/config.md`, `install.md`, `tui-chat-composer.md`, etc.
- `docs/logs/logging.md`, `debugging-logs.md`
- `docs/architects/orbit-code-testing-guide.md`
- `docs/migration/` and `docs/Learnings/` files
- Multiple CLAUDE.md and AGENTS.md files
- `shell-tool-mcp/README.md`

---

## Phase 11b: SDK Updates

**Not auto-generated — must be updated manually.**

- `sdk/typescript/src/codex.ts` (line 31): references `~/.codex/sessions`
- `sdk/typescript/README.md` (lines 100, 132): references `CODEX_API_KEY`, `CODEX_HOME`
- `sdk/typescript/tests/AGENTS.md` (line 34): references legacy paths

**Verify:** `cd sdk/typescript && pnpm test`

---

## Phase 12: Final Verification

```bash
just fmt
just fix                                    # Full workspace clippy
just write-config-schema                    # Regenerate config schema
just write-app-server-schema                # Already done in Phase 7
just write-app-server-schema --experimental
just test                                   # Full test suite
```

Scan for stragglers: search for `\.codex[^-]` to catch any remaining `.codex` directory references (excluding `codex-rs`, `.codex-plugin`, etc.).

---

## Risk Notes

| Risk | Level | Mitigation |
|------|-------|------------|
| Config loader stops scanning `.codex/` project dirs | Low | No users have `.codex/` dirs |
| Sandbox protection gaps | Medium | Test on macOS (seatbelt) to verify `.orbit` is protected |
| Keyring storage key change | Low | No users have stored credentials |
| Integration test breakage from missed renames | Low | Full `just test` at end catches all |
| Schema drift after v2.rs changes | Low | Schema regen + `cargo test -p orbit-code-app-server-protocol` |

---

## Acknowledged Edge Cases (No Action — Zero Users)

These are intentional consequences of the clean cut. All are acceptable given zero users:

1. **Orphaned `~/.codex` directories become invisible** — no migration warning printed. Acceptable: no users have them.
2. **Keyring entries under "Codex Auth" service become permanently inaccessible** — the `LEGACY_KEYRING_SERVICE` lookup is removed. Acceptable: no stored credentials exist.
3. **`CODEX_HOME` / `CODEX_API_KEY` env vars silently stop working** — no deprecation notice. Acceptable: no users rely on them.
4. **macOS managed preferences ID `com.openai.codex` remains unchanged** — deferred, separate MDM scope.
5. **Windows sandbox ACLs stop protecting `.codex` directories in existing projects** — only `.orbit` will be protected. Acceptable: no projects use `.codex`.
