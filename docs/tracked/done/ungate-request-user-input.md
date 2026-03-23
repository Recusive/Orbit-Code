# Plan: Ungate `request_user_input` — Make Available in All Modes

## Context

The `request_user_input` tool (lets the agent ask users multiple-choice questions) is currently gated to **Plan mode only**. An opt-in feature flag `DefaultModeRequestUserInput` extends it to Default mode, but defaults to `false`. The user wants the tool unconditionally available in **all collaboration modes** (Plan, Default, Execute, PairProgramming) with no feature flag needed.

This is a clean removal of the gating mechanism — not a flag flip. All dead code (mode checks, conditional messages, the feature flag, `CollaborationModesConfig` struct) gets removed to satisfy the codebase's zero-tolerance lint policy.

**Preserved behavior:** SubAgent exclusion (`!matches!(session_source, SessionSource::SubAgent(_))`) is orthogonal to mode gating and stays untouched.

## Key Changes

### 1. Remove protocol-level gate
- **`protocol/src/config_types.rs`** — Delete `ModeKind::allows_request_user_input()` method entirely (only returned `true` for `Plan`).

### 2. Retire feature flag (backward-compatible)
- **`core/src/features.rs`** — Change `DefaultModeRequestUserInput` spec from `Stage::UnderDevelopment, default_enabled: false` to `Stage::Removed, default_enabled: true`. Keeps config parsing working for users who set the key.

### 3. Simplify tool handler
- **`core/src/tools/handlers/request_user_input.rs`**:
  - Delete `request_user_input_is_available()`, `format_allowed_modes()`, `request_user_input_unavailable_message()`.
  - Simplify `request_user_input_tool_description()` — no parameter, return fixed string without mode restriction clause.
  - Simplify `RequestUserInputHandler` — remove `default_mode_request_user_input` field (becomes unit-like struct).
  - Remove mode-check block in `handle()` (lines 84-89).
  - Remove unused imports (`ModeKind`, `TUI_VISIBLE_COLLABORATION_MODES`).
- **`core/src/tools/handlers/request_user_input_tests.rs`** — Delete mode-availability tests, update description test.

### 4. Remove `CollaborationModesConfig` struct and plumbing
The struct has exactly one field (`default_mode_request_user_input`) — once removed, the struct is empty and dead.

- **`core/src/models_manager/collaboration_mode_presets.rs`**:
  - Delete `CollaborationModesConfig` struct.
  - Remove parameter from `builtin_collaboration_mode_presets()`, `default_preset()`, `default_mode_instructions()`.
  - Replace conditional `request_user_input_availability_message()` and `asking_questions_guidance_message()` with unconditional strings (tool always available, prefer using the tool).
- **`core/src/models_manager/collaboration_mode_presets_tests.rs`** — Update constructor calls, delete feature-flag-conditional test.

- **`core/src/models_manager/manager.rs`**:
  - Remove `collaboration_modes_config` field from `ModelsManager`.
  - Remove parameter from `new()`, `new_with_provider()`.
  - Delete `list_collaboration_modes_for_config()` — `list_collaboration_modes()` calls `builtin_collaboration_mode_presets()` directly.
  - Note: `with_provider_for_tests()` (line ~552) internally uses `CollaborationModesConfig::default()` — this is auto-fixed transitively when the struct is deleted, but verify the test helper compiles cleanly after removal.

- **`core/src/thread_manager.rs`** — Remove `CollaborationModesConfig` parameter from `ThreadManager::new()`.

- **`core/src/tools/spec.rs`**:
  - Remove `default_mode_request_user_input` field from `ToolsConfig`.
  - Simplify `create_request_user_input_tool()` (no param).
  - Simplify `RequestUserInputHandler` construction (no field).

### 5. Mirror in `tui_app_server`
- **`tui_app_server/src/model_catalog.rs`** — Same simplifications as `collaboration_mode_presets.rs`:
  - Remove `collaboration_modes_config` field from `ModelCatalog`, simplify `new()`.
  - Remove parameters from `builtin_collaboration_mode_presets()`, `default_preset()`, `default_mode_instructions()`.
  - Replace conditional availability/guidance functions with unconditional strings.

### 6. Update all consumer call sites
Remove `CollaborationModesConfig { ... }` construction from:
- `tui/src/app.rs`
- `tui_app_server/src/app.rs`
- `app-server/src/message_processor.rs`
- `app-server/src/orbit_code_message_processor.rs` — also simplify `normalize_turn_start_collaboration_mode()` to use `list_collaboration_modes()` instead of `list_collaboration_modes_for_config()`.
- `app-server-client/src/lib.rs` — **two** construction sites (line ~215 and ~1489), update both
- `mcp-server/src/message_processor.rs`

**Additional files found via grep** (constructor/import removal):
- `core/src/test_support.rs` — `ThreadManager` construction
- `core/src/orbit_code_tests.rs` — test setup
- `core/src/orbit_code_tests_guardian.rs` — guardian test setup
- `core/src/models_manager/manager_tests.rs` — `ModelsManager` construction
- `core/src/thread_manager_tests.rs` — `ThreadManager` construction
- `core/tests/suite/model_info_overrides.rs` — `ModelsManager` construction
- `core/src/codex.rs` — stale import of `CollaborationModesConfig` if present

**Closure step:** After all edits, run `grep -r "CollaborationModesConfig" codex-rs/` to confirm zero remaining references.

### 7. Update tests
- **`core/tests/suite/request_user_input.rs`** — Delete rejection tests (`assert_request_user_input_rejected` + 3 callers). Remove feature-flag enable block from `request_user_input_round_trip_for_mode`. Add round-trip tests for Default/Execute/PairProgramming modes.
- **`core/tests/suite/client.rs`**, **`core/tests/common/test_codex.rs`** — Remove `CollaborationModesConfig` from constructors.
- **`app-server/tests/suite/v2/turn_start.rs`** — Remove feature-flag config line. Also update any assertions on built-in Default-mode `developer_instructions` text — the content changes (stale "unavailable" text replaced with unconditional guidance). This is a deliberate app-server contract change.
- **`tui_app_server/src/chatwidget/tests.rs`**, **`tui/src/chatwidget/tests.rs`** — Remove `CollaborationModesConfig` from test setup.
- **`core/src/tools/spec_tests.rs`** — Update `create_request_user_input_tool(CollaborationModesConfig { ... })` calls (lines ~456, ~691-712) to no-arg `create_request_user_input_tool()`. Rewrite `request_user_input_description_reflects_default_mode_feature_flag` to assert unconditional description.
- **Add backward-compat assertion** — One test that sets `features.default_mode_request_user_input = true` in config and confirms it parses without error but does not change behavior (tool works identically with or without the key).

**App-server contract notes (deliberate changes):**
- `turn/start` built-in Default-mode instructions change content — any test asserting on that text needs updating.
- `experimentalFeature/list` will report `default_mode_request_user_input` as `removed` instead of `underDevelopment`. Verify any client or doc expectations around this metadata still hold. Add a targeted assertion if tests exist for the feature list response.

### 8. Remove template placeholders
**`core/templates/collaboration_mode/default.md`** — Remove the `{{REQUEST_USER_INPUT_AVAILABILITY}}` placeholder and its `## request_user_input availability` section entirely. The tool is now unconditionally available so mode-specific availability text is dead weight. Inline the `{{ASKING_QUESTIONS_GUIDANCE}}` content directly as static text: "prefer using the `request_user_input` tool rather than writing a multiple choice question as a textual assistant message." Remove the corresponding placeholder constants and `.replace()` calls from `collaboration_mode_presets.rs` and `tui_app_server/src/model_catalog.rs`.

## Verification

```bash
# Build & test bottom-up
cargo test -p orbit-code-protocol
cargo test -p orbit-code-core
cargo test -p orbit-code-tui
cargo test -p orbit-code-tui-app-server
cargo test -p orbit-code-app-server
cargo test -p orbit-code-app-server-client
cargo test -p orbit-code-mcp-server

# Lint
just fix -p orbit-code-protocol
just fix -p orbit-code-core
just fix -p orbit-code-tui
just fix -p orbit-code-tui-app-server
just fix -p orbit-code-app-server

# Schema + format
just write-config-schema
just fmt
```

## Execution Order

Build bottom-up to avoid breaking intermediate compilation states:
1. **Protocol layer** — `config_types.rs` (remove `allows_request_user_input()`)
2. **Core engine** — features, handler, spec, models_manager, thread_manager
3. **Consumer crates** — tui, tui_app_server, app-server, app-server-client, mcp-server
4. **Template** — `default.md` (remove placeholders, inline static text)
5. **Tests** — update/delete across all crates + grep closure for `CollaborationModesConfig`
6. **Schema + format** — `just write-config-schema`, `just fmt`

## Assumptions

- The `Feature::DefaultModeRequestUserInput` enum variant is kept as `Stage::Removed` (not deleted) for config backward compatibility — this is the established pattern in the codebase (see `Steer`, `CollaborationModes`, `SearchTool`).
- SubAgent exclusion is orthogonal and untouched.
- Template placeholders for mode-specific availability are removed (not kept with static substitutions) — if the text is unconditional, a placeholder adds complexity for no benefit.
- `plan.md` is left unchanged — it already has the correct `request_user_input` guidance with no mode restriction language.
- **No persisted session migration needed** — this is pre-launch, so no existing user sessions carry stale "request_user_input is unavailable" text. Local dev rollout artifacts can be cleared manually if encountered.

## Edge Cases

- **`features.default_mode_request_user_input = false` in config or `ThreadStartParams`:** Parses without error (`Stage::Removed` keeps the key in the schema). Has no runtime effect — all code paths that checked this feature are removed. The tool works regardless of the flag value.
- **Stale local dev rollout files:** Developers who ran pre-change builds may have local rollout files with old Default-mode instructions. These are dev artifacts, not user data. Starting a new session or deleting the local state directory clears them.
