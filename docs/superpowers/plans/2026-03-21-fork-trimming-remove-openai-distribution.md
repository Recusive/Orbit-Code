# Codex Fork Trimming — Remove OpenAI Distribution & Telemetry

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Surgically strip all OpenAI-specific distribution, telemetry, documentation, and unused IDE integration code from the forked Codex repo, leaving only the CLI binary, TUI, backend agent engine, and headless exec mode — ready to copy into the Orbit project.

**Architecture:** The repo is a Rust workspace with 68 crates. We keep ~40 crates (core engine, TUI, CLI, exec, auth, sandboxing, tools, config, local model support) and remove ~15 crate directories plus all non-Rust code (SDKs, npm wrapper, MCP server). Telemetry crates (`feedback`, `otel`) that phone home to OpenAI are removed and their callsites in kept crates are stubbed/deleted. The app-server family (`app-server`, `app-server-client`, `app-server-protocol`) is partially kept because `exec/` depends on `InProcessAppServerClient` for headless mode.

**Tech Stack:** Rust 1.93.0 (edition 2024), Cargo workspace, `just` task runner, ratatui, tokio

---

## Problem Statement

This repository is a fork of OpenAI's Codex CLI. We need it for one purpose: the terminal-based coding agent (CLI + TUI + backend engine). The fork contains significant infrastructure we don't need:

1. **OpenAI distribution** — npm wrapper (`codex-cli/`), Python/TypeScript SDKs (`sdk/`), release scripts
2. **Telemetry to OpenAI** — Sentry crash reporting (`feedback/`), OpenTelemetry metrics/traces (`otel/`) with hardcoded DSNs/endpoints
3. **IDE integration we've replaced** — `app-server` as a user-facing subcommand, `tui_app_server` (duplicate TUI), `mcp-server` (serving MCP to other tools)
4. **CI/CD for OpenAI** — Bazel build system (87 BUILD.bazel files), GitHub Actions, dev containers
5. **Non-Rust code** — TypeScript MCP server, Python SDK, npm packages, pnpm workspace

We must remove all of this **before** copying the repo into the Orbit project. The removal must be surgical because:
- Rust workspace members are wired through `Cargo.toml`
- Kept crates import types from removed crates (e.g., `core/` imports from `app-server-protocol/`)
- Telemetry calls are scattered across `core/`, `tui/`, `exec/`, `app-server/`

## Context & Constraints

- **What stays:** CLI binary, TUI (ratatui), core agent engine, exec (headless mode), auth (including Anthropic OAuth we added), sandboxing, config, hooks, state, all tools, local model support (LMStudio, Ollama), MCP client support
- **What goes:** Everything else — SDKs, npm, telemetry, MCP server mode, app-server as user-facing feature, Bazel, docs, scripts
- **Critical dependency chain:** `exec/` → `app-server-client/` → `app-server/` → `app-server-protocol/`. We CANNOT remove `app-server/` without rewriting `exec/`. For now, we keep the chain but remove `app-server` as a user-facing CLI subcommand.
- **`app-server-protocol/`** contains shared types used by `core/` (ConfigLayerSource, MCP elicitation types, AppInfo) and `tui/`. Must be kept regardless.
- **`core/` has 28 files** importing from `otel`/`feedback` — these are metrics recording, trace context propagation, and telemetry initialization scattered across auth, state, tasks, tools, memories, and session management. Removal requires deleting those callsites and any functions that exist solely to pass telemetry objects.
- **`codex-experimental-api-macros/`** must be KEPT — `app-server-protocol` depends on it (line 18 of its Cargo.toml) for the `ExperimentalApi` derive macro. Removing it breaks the entire workspace.
- **`feedback/`** uses Sentry with a hardcoded DSN — this sends user data to OpenAI. Must go.
- **After trimming**, `cargo build -p orbit-code` must succeed and `just codex` must launch the TUI.

## Decision: What Happens to `exec/`

`exec/` (headless mode: `orbit-code exec "fix the bug"`) communicates with core through `InProcessAppServerClient`, which embeds a full `app-server` in-process. This means keeping `exec/` requires keeping:
- `app-server/` (the server logic)
- `app-server-client/` (the client wrapper)
- `app-server-protocol/` (shared types)

**Recommended:** Keep all three for now. Remove `app-server` as a user-facing CLI subcommand (users won't run `orbit-code app-server`). A future project can rewrite `exec/` to talk to core directly like `tui/` does, at which point the app-server family can be fully removed.

---

## Phase 1: Delete Top-Level Directories & Files (Safe — No Rust Dependencies)

### Task 1.1: Delete non-Rust directories

**Files:**
- Delete: `sdk/` (entire directory)
- Delete: `shell-tool-mcp/` (entire directory)
- Delete: `codex-cli/` (entire directory)
- Delete: `scripts/` (entire directory)
- Delete: `tools/` (entire directory)
- Delete: `third_party/` (entire directory)
- Delete: `patches/` (entire directory)
- Delete: `reference/` (entire directory)
- Delete: `lancedb/` (entire directory)
- Delete: `reviews/` (entire directory)
- Delete: `.devcontainer/` (entire directory)
- Delete: `.codex/` (entire directory)

- [ ] **Step 1: Delete directories**

```bash
rm -rf sdk/ shell-tool-mcp/ codex-cli/ scripts/ tools/ third_party/ patches/ reference/ lancedb/ reviews/ .devcontainer/ .codex/
```

- [ ] **Step 2: Verify no Cargo references broke**

```bash
cd codex-rs && cargo check -p orbit-code 2>&1 | head -20
```

Expected: Should still compile (none of these were Rust workspace members except indirectly).

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "chore: remove non-Rust directories (SDKs, npm, scripts, tools, docs)"
```

### Task 1.2: Delete top-level config files for removed systems

**Files:**
- Delete: `pnpm-workspace.yaml`, `pnpm-lock.yaml`, `package.json`
- Delete: `.npmrc`, `.prettierrc.toml`, `.prettierignore`
- Delete: `MODULE.bazel`, `MODULE.bazel.lock`, `BUILD.bazel`, `.bazelrc`, `.bazelignore`, `.bazelversion`, `defs.bzl`, `rbe.bzl`, `workspace_root_test_launcher.bat.tpl`
- Delete: `flake.nix`, `flake.lock`
- Delete: `cliff.toml`, `announcement_tip.toml`
- Delete: `CONTRIBUTING.md`, `SECURITY.md`, `CHANGELOG.md`, `NOTICE`, `README.md`, `AGENTS.md`

- [ ] **Step 1: Delete files**

```bash
rm -f pnpm-workspace.yaml pnpm-lock.yaml package.json .npmrc .prettierrc.toml .prettierignore
rm -f MODULE.bazel MODULE.bazel.lock BUILD.bazel .bazelrc .bazelignore .bazelversion defs.bzl rbe.bzl workspace_root_test_launcher.bat.tpl
rm -f flake.nix flake.lock cliff.toml announcement_tip.toml
rm -f CONTRIBUTING.md SECURITY.md CHANGELOG.md NOTICE README.md AGENTS.md
```

- [ ] **Step 2: Commit**

```bash
git add -A && git commit -m "chore: remove Bazel, npm, Nix, and OpenAI doc files"
```

---

## Phase 2: Delete Removable Rust Crates (Directories)

These crates are either unused by any kept crate, or are only used by other crates also being removed.

### Task 2.1: Delete cleanly removable crate directories

**Files:**
- Delete: `codex-rs/tui_app_server/` (duplicate TUI — we have `tui/`)
- Delete: `codex-rs/mcp-server/` (serving MCP — we consume, not serve)
- Delete: `codex-rs/exec-server/` (execution server variant)
- Delete: `codex-rs/feedback/` (Sentry telemetry to OpenAI)
- Delete: `codex-rs/otel/` (OpenTelemetry to OpenAI)
- Delete: `codex-rs/debug-client/` (app-server debug tool)
- Delete: `codex-rs/app-server-test-client/` (test utility for app-server)
- Delete: `codex-rs/execpolicy-legacy/` (deprecated)
- Delete: `codex-rs/test-macros/` (test utility — only used by core dev-deps)
- Delete: `codex-rs/stdio-to-uds/` (app-server stdio transport)
- **KEEP: `codex-rs/codex-experimental-api-macros/`** — `app-server-protocol` depends on this for the `ExperimentalApi` derive macro. Cannot be removed without rewriting `app-server-protocol`.

- [ ] **Step 1: Delete crate directories**

```bash
cd codex-rs
rm -rf tui_app_server/ mcp-server/ exec-server/ feedback/ otel/ debug-client/ app-server-test-client/ execpolicy-legacy/ test-macros/ stdio-to-uds/
```

- [ ] **Step 2: Commit**

```bash
git add -A && git commit -m "chore: remove 10 unused Rust crates (telemetry, app-server extras, legacy)"
```

### Task 2.2: Update workspace Cargo.toml — remove deleted members

**Files:**
- Modify: `codex-rs/Cargo.toml` (workspace members list and workspace.dependencies)

- [ ] **Step 1: Remove deleted crates from `[workspace] members`**

Remove these entries from the `members = [...]` array in `codex-rs/Cargo.toml`:

```
"app-server-test-client"
"debug-client"
"exec-server"
"execpolicy-legacy"
"feedback"
"mcp-server"
"otel"
"stdio-to-uds"
"tui_app_server"
"test-macros"
```

**DO NOT remove `"codex-experimental-api-macros"` — `app-server-protocol` depends on it.**

- [ ] **Step 2: Remove deleted crates from `[workspace.dependencies]`**

Remove these lines from `[workspace.dependencies]` (use exact names — grep the file to find each one):

```toml
orbit-code-feedback = { path = "feedback" }
orbit-code-otel = { path = "otel" }
orbit-code-tui-app-server = { path = "tui_app_server" }
orbit-code-test-macros = { path = "test-macros" }
orbit-code-stdio-to-uds = { path = "stdio-to-uds" }
```

Also remove the `mcp_test_support` entry (line 147):
```toml
mcp_test_support = { path = "mcp-server/tests/common" }
```

**Note:** `exec-server`, `debug-client`, and `execpolicy-legacy` do NOT have entries in `[workspace.dependencies]` — they are members only. No action needed for those in this step.

- [ ] **Step 3: Remove orphaned external crate dependencies**

After removing `otel/` and `feedback/`, these external crates become unused workspace-wide. Remove them from `[workspace.dependencies]`:

```toml
opentelemetry = "0.31.0"
opentelemetry-appender-tracing = "0.31.0"
opentelemetry-otlp = "0.31.0"
opentelemetry-semantic-conventions = "0.31.0"
opentelemetry_sdk = "0.31.0"
sentry = "0.46.0"
tracing-opentelemetry = "0.32.0"
```

**Verify first:** grep the entire `codex-rs/` for each crate name in remaining Cargo.toml files to confirm no kept crate uses them. Some (like `opentelemetry_sdk`) may be used in dev-dependencies of kept crates — remove those dev-dep lines too.

- [ ] **Step 4: Verify workspace parses**

```bash
cargo metadata --format-version=1 --no-deps 2>&1 | head -5
```

Expected: Valid JSON output, no "failed to load manifest" errors.

- [ ] **Step 4: Commit**

```bash
git add codex-rs/Cargo.toml && git commit -m "chore: remove deleted crates from workspace members and dependencies"
```

---

## Phase 3: Strip Telemetry From Kept Crates

The `feedback` and `otel` crates are imported by `core/`, `tui/`, `exec/`, `app-server/`, and `app-server-client/`. Every import and callsite must be removed or stubbed.

### Task 3.1: Strip `feedback` and `otel` from `core/`

**This is the largest single task.** The `core/` crate has 28 source files importing `orbit_code_otel` or `orbit_code_feedback`. Every import and callsite must be removed.

**Files:**
- Modify: `codex-rs/core/Cargo.toml` — remove `orbit-code-otel` from `[dependencies]` and `[dev-dependencies]`, remove `orbit-code-test-macros` from `[dev-dependencies]`
- Modify (telemetry-heavy — SessionTelemetry parameters/fields):
  - `codex-rs/core/src/codex.rs` — session construction passes `SessionTelemetry`
  - `codex-rs/core/src/otel_init.rs` — **likely can delete entirely** (otel initialization module)
  - `codex-rs/core/src/client.rs` — trace context propagation
  - `codex-rs/core/src/client_tests.rs` — `SessionTelemetry` in test setup
  - `codex-rs/core/src/tasks/mod.rs` — metric recording (E2E, token, tool, proxy)
  - `codex-rs/core/src/tasks/mod_tests.rs` — otel test setup
  - `codex-rs/core/src/turn_timing.rs` — TTFM/TTFT duration metrics
  - `codex-rs/core/src/features.rs` — `SessionTelemetry` parameter
  - `codex-rs/core/src/memory_trace.rs` — `SessionTelemetry` parameter
  - `codex-rs/core/src/tools/orchestrator.rs` — `ToolDecisionSource`
- Modify (lighter touch — scattered telemetry calls):
  - `codex-rs/core/src/auth.rs` — telemetry in auth flows
  - `codex-rs/core/src/auth_env_telemetry.rs` — **likely can delete entirely** (auth telemetry module)
  - `codex-rs/core/src/state/service.rs` — telemetry in state operations
  - `codex-rs/core/src/state/turn.rs` — telemetry in turn tracking
  - `codex-rs/core/src/memories/phase1.rs` — telemetry in memory operations
  - `codex-rs/core/src/memories/phase2.rs` — same
  - `codex-rs/core/src/models_manager/manager.rs` — telemetry in model management
  - `codex-rs/core/src/mcp_connection_manager.rs` — telemetry in MCP connections
  - `codex-rs/core/src/shell_snapshot.rs` — telemetry in shell snapshots
  - `codex-rs/core/src/rollout/metadata.rs` — telemetry in rollout
  - `codex-rs/core/src/agent/guards.rs` — telemetry in agent guards
  - `codex-rs/core/src/skills/injection.rs` — telemetry in skills
  - `codex-rs/core/src/external_agent_config.rs` — telemetry in agent config
  - `codex-rs/core/src/session_startup_prewarm.rs` — telemetry in session prewarm
  - `codex-rs/core/src/windows_sandbox.rs` — telemetry in Windows sandbox
  - `codex-rs/core/src/exec.rs` — telemetry in exec
  - `codex-rs/core/src/util.rs` — telemetry utilities
  - `codex-rs/core/src/orbit_code_tests.rs` — telemetry in tests
- Modify: `codex-rs/core/tests/suite/apply_patch_cli.rs` — remove `orbit_code_test_macros`

- [ ] **Step 1: Remove dependencies from Cargo.toml**

In `codex-rs/core/Cargo.toml`:
- Remove `orbit-code-otel = { workspace = true }` from `[dependencies]`
- Remove `orbit-code-otel = { workspace = true, features = [...] }` from `[dev-dependencies]`
- Remove `orbit-code-test-macros = { workspace = true }` from `[dev-dependencies]`
- Remove any `opentelemetry*` or `tracing-opentelemetry` dev-dependencies

- [ ] **Step 2: Delete dedicated telemetry modules**

These modules exist solely for telemetry and can likely be deleted entirely:
- `codex-rs/core/src/otel_init.rs` — otel initialization
- `codex-rs/core/src/auth_env_telemetry.rs` — auth telemetry

Remove their `mod` declarations from the parent module.

- [ ] **Step 3: Fix all 28 source files**

**General strategy for each file:**
1. Remove `use orbit_code_otel::*` imports
2. Remove any function parameters of type `SessionTelemetry`, `&SessionTelemetry`, `Option<SessionTelemetry>`, `Arc<SessionTelemetry>`, etc.
3. Remove calls like `session_telemetry.record_*()`, `metrics_client.record_*()`, etc.
4. If a function's only purpose was telemetry, delete the function and remove calls to it
5. If `SessionTelemetry` was passed through a struct field, remove the field and all construction sites
6. If a function accepts `SessionTelemetry` as a parameter, remove the parameter and fix all callers (this cascades — start from leaf functions and work up)

**The cascade pattern:** `SessionTelemetry` is likely threaded through `Session` → `Task` → individual tool/handler functions. Removing it from `Session` (in `codex.rs`) will cascade errors through `tasks/mod.rs`, `client.rs`, `tools/orchestrator.rs`, etc. Work iteratively: remove from Cargo.toml, then `cargo check`, fix each error, repeat.

- [ ] **Step 4: Fix test file**

In `core/tests/suite/apply_patch_cli.rs`:
- Replace `use orbit_code_test_macros::large_stack_test;` with nothing
- Replace `#[large_stack_test]` attribute with `#[tokio::test]` (or `#[test]` if sync)

- [ ] **Step 4: Verify core compiles**

```bash
cargo check -p orbit-code-core 2>&1 | head -30
```

Fix any remaining compilation errors iteratively.

- [ ] **Step 5: Run core tests**

```bash
cargo test -p orbit-code-core 2>&1 | tail -20
```

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -m "chore: strip otel and feedback telemetry from core crate"
```

### Task 3.2: Strip `feedback` and `otel` from `tui/`

**Files:**
- Modify: `codex-rs/tui/Cargo.toml` — remove `orbit-code-feedback`, `orbit-code-otel`, `orbit-code-tui-app-server`
- Modify: `codex-rs/tui/src/bottom_pane/feedback_view.rs` — delete or gut the feedback submission UI
- Modify: `codex-rs/tui/src/history_cell.rs` — remove `RuntimeMetricsSummary` / `RuntimeMetricTotals` rendering
- Modify: `codex-rs/tui/src/chatwidget.rs` — remove otel metrics display, `SessionTelemetry` usage
- Modify: `codex-rs/tui/src/app.rs` — remove otel initialization (`SessionTelemetry`, `TelemetryAuthMode`), feedback setup
- Modify: `codex-rs/tui/src/chatwidget/tests.rs` — remove `SessionTelemetry` in test setup
- Modify: `codex-rs/tui/src/debug_config.rs` — remove `ConfigLayerSource` if it came from app-server-protocol (check: this type might still exist via the kept `app-server-protocol`)

- [ ] **Step 1: Remove dependencies from Cargo.toml**

In `codex-rs/tui/Cargo.toml`:
- Remove `orbit-code-feedback = { workspace = true }`
- Remove `orbit-code-otel = { workspace = true }`
- Remove `orbit-code-tui-app-server = { workspace = true }` (if present — tui_app_server is deleted)

- [ ] **Step 2: Remove feedback UI**

`feedback_view.rs` is the feedback submission dialog. Either:
- Delete the file entirely and remove its `mod` declaration from the parent module
- Or gut it to a no-op if the TUI framework expects the module to exist

Also remove any "Submit Feedback" keybinding or menu item in `app.rs`.

- [ ] **Step 3: Remove otel metrics rendering**

In `history_cell.rs` and `chatwidget.rs`:
- Remove imports of `RuntimeMetricsSummary`, `RuntimeMetricTotals`
- Remove any UI widgets that display these metrics
- If metrics were shown in a status bar or debug panel, remove those render blocks

In `app.rs`:
- Remove `SessionTelemetry` initialization
- Remove `TelemetryAuthMode` setup
- Remove otel provider creation / shutdown

- [ ] **Step 4: Verify TUI compiles**

```bash
cargo check -p orbit-code-tui 2>&1 | head -30
```

Fix iteratively.

- [ ] **Step 5: Commit**

```bash
git add -A && git commit -m "chore: strip otel and feedback from TUI crate"
```

### Task 3.3: Strip `feedback` and `otel` from `exec/`

**Files:**
- Modify: `codex-rs/exec/Cargo.toml` — remove `orbit-code-feedback`, `orbit-code-otel`
- Modify: `codex-rs/exec/src/lib.rs` — remove `CodexFeedback`, `set_parent_from_context`, `traceparent_context_from_env`, `set_parent_from_w3c_trace_context`

- [ ] **Step 1: Remove dependencies from Cargo.toml**

- Remove `orbit-code-feedback = { workspace = true }`
- Remove `orbit-code-otel = { workspace = true }`

- [ ] **Step 2: Fix lib.rs**

- Remove `use orbit_code_feedback::CodexFeedback;`
- Remove `use orbit_code_otel::*` imports
- Remove `CodexFeedback` construction and any `feedback.upload_*()` calls
- Remove otel context propagation calls
- Keep all actual exec session logic

- [ ] **Step 3: Verify exec compiles**

```bash
cargo check -p orbit-code-exec 2>&1 | head -30
```

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "chore: strip otel and feedback from exec crate"
```

### Task 3.4: Strip `feedback` and `otel` from `app-server/` and `app-server-client/`

These crates are kept (exec depends on them) but still import feedback/otel.

**Files:**
- Modify: `codex-rs/app-server/Cargo.toml` — remove `orbit-code-feedback`, `orbit-code-otel`
- Modify: `codex-rs/app-server-client/Cargo.toml` — remove `orbit-code-feedback`
- Modify: source files in both crates referencing feedback/otel

- [ ] **Step 1: Remove dependencies from both Cargo.toml files**

- [ ] **Step 2: Grep and fix source files**

```bash
grep -rn "orbit_code_feedback\|orbit_code_otel" codex-rs/app-server/src/ codex-rs/app-server-client/src/
```

Remove all imports and callsites found.

- [ ] **Step 3: Verify both compile**

```bash
cargo check -p orbit-code-app-server -p orbit-code-app-server-client 2>&1 | head -30
```

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "chore: strip otel and feedback from app-server and app-server-client"
```

---

## Phase 4: Strip Dead CLI Subcommands

### Task 4.1: Remove dead subcommands from CLI

**Files:**
- Modify: `codex-rs/cli/src/main.rs` — remove subcommands for deleted crates
- Modify: `codex-rs/cli/Cargo.toml` — remove dependencies on deleted crates
- Delete: `codex-rs/cli/src/app_cmd.rs` — macOS desktop app launcher (downloads OpenAI's app)
- Delete: `codex-rs/cli/src/desktop_app/` — supporting module for above

- [ ] **Step 1: Remove dead dependencies from cli/Cargo.toml**

Remove:
- `orbit-code-mcp-server = { workspace = true }`
- `orbit-code-stdio-to-uds = { workspace = true }`
- `orbit-code-tui-app-server = { workspace = true }`
- `orbit-code-app-server-test-client = { workspace = true }`

Keep (still needed by exec integration):
- `orbit-code-app-server = { workspace = true }`
- `orbit-code-app-server-protocol = { workspace = true }`

- [ ] **Step 2: Remove subcommand variants from `Subcommand` enum in main.rs**

Remove these variants:
```rust
McpServer,                              // mcp-server crate deleted
App(app_cmd::AppCommand),               // macOS OpenAI app launcher
StdioToUds(StdioToUdsCommand),          // stdio-to-uds crate deleted
```

Remove from `DebugSubcommand`:
```rust
AppServer(DebugAppServerCommand),       // debug-client deleted
```

Consider removing but OPTIONAL (app-server still exists, just hidden):
```rust
AppServer(AppServerCommand),            // keep but mark #[clap(hide = true)]
```

- [ ] **Step 3: Remove dispatch arms in `cli_main()`**

Remove the match arms that dispatched to deleted subcommands:
- `Subcommand::McpServer => { orbit_code_mcp_server::run_main().await }`
- `Subcommand::StdioToUds(cmd) => { orbit_code_stdio_to_uds::run(...) }`
- `Subcommand::App(cmd) => { ... }` (macOS only)
- Debug app-server send-message-v2 dispatch

- [ ] **Step 4: Remove the `InteractiveRemoteOptions` TUI-app-server path**

In `cli_main()`, there's logic around lines 1052-1138 that dispatches to `orbit_code_tui_app_server::run_main()` when remote options are provided. Remove this entire code path. The `InteractiveRemoteOptions` struct and `--remote` flag can be removed from `MultitoolCli`.

- [ ] **Step 5: Delete dead source files**

```bash
rm -f codex-rs/cli/src/app_cmd.rs
rm -rf codex-rs/cli/src/desktop_app/
```

- [ ] **Step 6: Remove dead module declarations**

In `main.rs`, remove:
```rust
#[cfg(target_os = "macos")]
mod app_cmd;
#[cfg(target_os = "macos")]
mod desktop_app;
```

- [ ] **Step 7: Verify CLI compiles**

```bash
cargo check -p orbit-code 2>&1 | head -30
```

- [ ] **Step 8: Commit**

```bash
git add -A && git commit -m "chore: remove dead CLI subcommands (mcp-server, app, stdio-to-uds, debug)"
```

---

## Phase 5: Delete All BUILD.bazel Files

### Task 5.1: Remove Bazel build system

**Files:**
- Delete: All `BUILD.bazel` files across `codex-rs/` (60+ files)
- Delete: `codex-rs/vendor/` (Bazel-specific vendored binaries)
- Delete: `codex-rs/docs/bazel.md` (if exists)

- [ ] **Step 1: Delete all BUILD.bazel files**

```bash
find codex-rs/ -name "BUILD.bazel" -delete
```

- [ ] **Step 2: Delete Bazel vendor directory**

```bash
rm -rf codex-rs/vendor/
```

- [ ] **Step 3: Verify Cargo still works**

```bash
cargo check -p orbit-code 2>&1 | head -10
```

Expected: Cargo ignores BUILD.bazel files — this should be a no-op for compilation.

- [ ] **Step 4: Commit**

```bash
git add -A && git commit -m "chore: remove all Bazel build files"
```

---

## Phase 6: Clean Up Miscellaneous Files

### Task 6.1: Remove remaining OpenAI-specific files inside codex-rs/

**Files:**
- Delete: `codex-rs/node-version.txt`
- Delete: `codex-rs/config.md` (generated docs)
- Delete: `codex-rs/AGENTS.md` (OpenAI agent instructions)
- Delete: `codex-rs/docs/` (if contains only Bazel/OpenAI docs; check first)
- Delete: `codex-rs/scripts/` (check if any are needed for `just` commands)
- Modify: `codex-rs/deny.toml` — keep if it's cargo-deny config, remove if Bazel-related

- [ ] **Step 1: Review and delete**

Check each file/directory before deleting. Keep anything referenced by the `justfile` or `Cargo.toml`.

- [ ] **Step 2: Clean up `.gitignore`**

Remove entries for deleted systems (Bazel output, npm artifacts, Python venvs, etc.).

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "chore: remove remaining OpenAI-specific config files"
```

### Task 6.2: Remove `announcement_tip.toml` and its reader

**Files:**
- Delete: `announcement_tip.toml` (root level — OpenAI release announcements)
- Modify: Any code in `tui/` or `cli/` that reads this file

- [ ] **Step 1: Find references**

```bash
grep -rn "announcement_tip\|announcement" codex-rs/tui/src/ codex-rs/cli/src/
```

- [ ] **Step 2: Remove the file and all code that reads it**

- [ ] **Step 3: Commit**

```bash
git add -A && git commit -m "chore: remove OpenAI announcement tip system"
```

---

## Phase 7: Full Verification

### Task 7.1: Compile the entire workspace

- [ ] **Step 1: Full workspace check**

```bash
cd codex-rs && cargo check 2>&1
```

Fix any remaining compilation errors. Common issues:
- Orphaned `use` statements referencing deleted types
- Struct fields of deleted types (e.g., `SessionTelemetry`)
- Function signatures that accepted telemetry parameters
- Test helpers that imported deleted crates

- [ ] **Step 2: Format**

```bash
just fmt
```

- [ ] **Step 3: Clippy**

```bash
just fix 2>&1 | tail -30
```

- [ ] **Step 4: Commit fixes**

```bash
git add -A && git commit -m "fix: resolve compilation errors from crate removal"
```

### Task 7.2: Run tests on critical crates

- [ ] **Step 1: Test core**

```bash
cargo test -p orbit-code-core 2>&1 | tail -20
```

- [ ] **Step 2: Test TUI**

```bash
cargo test -p orbit-code-tui 2>&1 | tail -20
```

- [ ] **Step 3: Test exec**

```bash
cargo test -p orbit-code-exec 2>&1 | tail -20
```

- [ ] **Step 4: Test CLI**

```bash
cargo test -p orbit-code 2>&1 | tail -20
```

- [ ] **Step 5: Accept any snapshot changes**

```bash
cargo insta accept -p orbit-code-tui
```

- [ ] **Step 6: Commit test fixes**

```bash
git add -A && git commit -m "fix: update tests for crate removal"
```

### Task 7.3: Smoke test the binary

- [ ] **Step 1: Run from source**

```bash
just codex --help
```

Expected: Shows help with subcommands (exec, login, logout, resume, fork, mcp, sandbox, completion). Should NOT show: app-server, mcp-server, app.

- [ ] **Step 2: Launch TUI**

```bash
just codex
```

Expected: TUI launches, chat interface renders, input works. Exit with Ctrl-C.

- [ ] **Step 3: Test headless mode**

```bash
echo "say hello" | just codex exec --quiet
```

Expected: Agent responds (requires valid auth).

- [ ] **Step 4: Final commit**

```bash
git add -A && git commit -m "chore: fork trimming complete — CLI, TUI, and backend verified"
```

---

## Summary: What's Removed vs What's Kept

### Removed (~28 crate dirs + all non-Rust code)

| Category | Removed Items |
|---|---|
| **Top-level dirs** | `sdk/`, `shell-tool-mcp/`, `codex-cli/`, `scripts/`, `tools/`, `third_party/`, `patches/`, `reference/`, `lancedb/`, `reviews/`, `.devcontainer/`, `.codex/`, `docs/` (most of it) |
| **Rust crates** | `tui_app_server/`, `mcp-server/`, `exec-server/`, `feedback/`, `otel/`, `debug-client/`, `app-server-test-client/`, `execpolicy-legacy/`, `codex-experimental-api-macros/`, `test-macros/`, `stdio-to-uds/` |
| **Build systems** | All BUILD.bazel (87 files), MODULE.bazel, Bazel config |
| **Config files** | npm, pnpm, Nix, prettier, cliff, announcement_tip |
| **CLI subcommands** | `mcp-server`, `app`, `debug app-server`, `stdio-to-uds` |
| **Functionality** | Sentry crash reports, OpenTelemetry to OpenAI, npm distribution, Python/TS SDKs, MCP server mode, macOS app launcher, remote TUI mode |

### Kept (~40 crate dirs)

| Category | Kept Items |
|---|---|
| **Binary + UI** | `cli/`, `tui/`, `exec/` |
| **Engine** | `core/`, `protocol/`, `app-server-protocol/`, `app-server/`, `app-server-client/` |
| **Auth** | `login/`, `secrets/`, `keyring-store/` |
| **Config** | `config/`, `hooks/`, `execpolicy/` |
| **State** | `state/` |
| **Tools** | `shell-command/`, `apply-patch/`, `file-search/`, `skills/`, `artifacts/`, `rmcp-client/` |
| **Sandbox** | `linux-sandbox/`, `windows-sandbox-rs/`, `process-hardening/`, `shell-escalation/` |
| **Models** | `anthropic/`, `lmstudio/`, `ollama/` |
| **Infrastructure** | `codex-client/`, `codex-api/`, `ansi-escape/`, `arg0/`, `environment/`, `network-proxy/`, `async-utils/`, `package-manager/` |
| **Utils** | All 19 crates in `utils/` |

### Future Work (Not In This Plan)

- **Rewrite `exec/` to talk to core directly** — eliminates dependency on `app-server/`, `app-server-client/`, and `app-server-protocol/`. This is a separate project.
- **Move shared types from `app-server-protocol/` to `protocol/`** — cleaner separation. Do after exec rewrite.
- **Add Open Router / VLLM provider crates** — new model providers alongside `anthropic/`, `lmstudio/`, `ollama/`.
- **Replace OpenAI Responses API with multi-provider abstraction** — the core engine currently speaks OpenAI's Responses API format. Needs abstraction layer for Claude, Llama, etc.
