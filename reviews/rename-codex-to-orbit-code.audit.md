# Plan Audit: Rename codex -> orbit-code Implementation Plan (v3 - Post-Audit)

**Date**: 2026-03-19
**Plan Document**: `docs/orbit/plans/rename-codex-to-orbit-code.md`
**Branch**: `main`

## Plan Summary

The updated plan is a clear improvement over the previous version. It now separates internal naming from compatibility-sensitive surfaces, explicitly preserves wire protocol identifiers like `codex/sandbox-state`, avoids touching the sandbox env-var contract, and adds an initial migration idea for moving user-home state from `~/.codex` to `~/.orbit-code`.

The remaining problem is execution quality: the compatibility framing is better, but the proposed implementation still has several concrete gaps. The rename script will not run as written, the migration strategy only covers one subset of `.codex` state, the keyring fallback step points at the wrong crate, and the public env/package migration story is still incomplete.

## Files Reviewed

| File | Role | Risk |
| ---- | ---- | ---- |
| `AGENTS.md` | Root repo rules for sandbox env vars, schema/test workflow, and full-suite approval requirements | High |
| `docs/orbit/plans/rename-codex-to-orbit-code.md` | Updated rename plan | High |
| `justfile` | Canonical commands for config/app-server schema and Bazel lock maintenance | High |
| `codex-rs/core/src/spawn.rs` | Protected sandbox env-var contract | High |
| `codex-rs/core/src/mcp_connection_manager.rs` | MCP `codex/sandbox-state` capability/method that must stay stable | High |
| `codex-rs/codex-api/src/endpoint/realtime_websocket/protocol_v2.rs` | Realtime tool name `codex` wire contract | High |
| `codex-rs/codex-api/src/endpoint/realtime_websocket/methods_v2.rs` | Realtime tool declaration | High |
| `codex-rs/mcp-server/src/codex_tool_config.rs` | Public MCP tool schema/name | High |
| `codex-rs/mcp-server/src/message_processor.rs` | MCP tool dispatch for `codex` / `codex-reply` | High |
| `codex-rs/core/src/config_loader/mod.rs` | Project/repo `.codex/config.toml` discovery semantics | High |
| `codex-rs/utils/home-dir/src/lib.rs` | `CODEX_HOME` / `~/.codex` resolution | High |
| `codex-rs/utils/home-dir/AGENTS.md` | Local rules for the home-dir crate | Medium |
| `codex-rs/core/src/auth.rs` | `OPENAI_API_KEY` and `CODEX_API_KEY` env behavior | High |
| `codex-rs/core/src/auth/storage.rs` | Keyring service name and persisted auth file handling | High |
| `codex-rs/keyring-store/src/lib.rs` | Actual responsibility boundary for keyring store crate | Medium |
| `codex-rs/app-server-client/src/lib.rs` | Public in-process client option `enable_codex_api_key_env` | Medium |
| `codex-rs/app-server/src/in_process.rs` | Embedded app-server auth/env compatibility surface | Medium |
| `codex-rs/cli/Cargo.toml` | Current CLI package/bin naming | Medium |
| `sdk/typescript/src/exec.ts` | TypeScript SDK package/env compatibility and binary launch env | High |
| `sdk/typescript/src/codex.ts` | SDK docs mentioning persisted threads under `~/.codex` | Medium |
| `sdk/python/_runtime_setup.py` | Python runtime package/release download contract | High |
| `sdk/python/src/codex_app_server/client.py` | Python runtime package lookup and app-server launch contract | High |
| `sdk/python-runtime/pyproject.toml` | Published runtime wheel name/package path | High |
| `sdk/python-runtime/src/codex_cli_bin/__init__.py` | Runtime binary lookup contract | High |
| `codex-cli/bin/codex.js` | npm binary/package resolution | High |
| `codex-cli/scripts/build_npm_package.py` | npm release package expansion and naming | High |
| `scripts/stage_npm_packages.py` | release orchestration against GitHub/openai naming | High |

_Risk: High (core contract or shared subsystem), Medium (feature code or adapter), Low (leaf utility, tests, docs)_

## Verdict: NEEDS REWORK

The v3 plan is materially closer, but it is still not implementation-ready. The architecture now respects several key compatibility boundaries, yet the proposed script and migration steps still contain blocking defects and incomplete compatibility coverage for project-local config, auth/keyring migration, and public env/package behavior.

## Critical Issues (Must Fix Before Implementation)

| # | Section | Problem | Recommendation |
| - | ------- | ------- | -------------- |
| 1 | 2.4 | The rename script is mechanically broken. It defines `sedi()` in the parent shell (`docs/orbit/plans/rename-codex-to-orbit-code.md:129-136`) but then invokes it inside fresh `bash -c` subprocesses spawned by `xargs` across nearly every phase (`:143-146`, `:163-166`, `:179-182`, `:353-358`, `:375-380`, etc.). Shell functions are not available in those subprocesses unless exported, so the script will fail with `sedi: command not found` before doing the rename. | Replace the `xargs ... bash -c` pattern with `find ... -exec`, export the function explicitly, or drop the function entirely and use a standalone helper script/tool. Validate the script end-to-end on both Linux and macOS before treating it as the primary migration mechanism. |
| 2 | 2.1 | The new migration strategy only addresses the user home directory, but the plan also renames project-local `.codex/` paths globally (`docs/orbit/plans/rename-codex-to-orbit-code.md:37`, `:271-279`). Current config loading explicitly reads `./.codex/config.toml` from parent directories and repo root (`codex-rs/core/src/config_loader/mod.rs:100-103`). The broader repo also relies on `.codex/skills`, `.codex/.env`, session/history docs, and path strings throughout SDK/docs/tests. A home-dir migration shim does not preserve existing repo-local config behavior. | Treat repo-local `.codex/` as a separate migration surface. Either keep dual-read support for both `.codex` and `.orbit-code` at project scope for at least one transition period, or write a dedicated repo-config migration plan with compatibility semantics and rollback. Do not globally rewrite `.codex/` literals until the loader behavior is updated first. |
| 3 | 2.2 | The keyring migration step points to the wrong place. The plan says to add `Orbit Code Auth` / `Codex Auth` fallback handling in `codex-rs/keyring-store/src/lib.rs` (`docs/orbit/plans/rename-codex-to-orbit-code.md:551-558`), but that crate is only a generic store abstraction. The actual service name is currently defined in `codex-rs/core/src/auth/storage.rs:135`. Implementing the plan as written would not migrate auth lookups at all. | Move this step into `codex-rs/core/src/auth/storage.rs`, where `KEYRING_SERVICE` actually lives, and define the exact read/write/delete fallback behavior. Also specify whether the account key derived from `codex_home` changes when the home path changes, because service-name fallback alone may not be sufficient. |
| 4 | 2.3 | The plan still treats `CODEX_API_KEY` as an internal rename target (`docs/orbit/plans/rename-codex-to-orbit-code.md:34`), but current code exposes it across multiple surfaces: `codex-rs/core/src/auth.rs:377-389`, app-server embedder flags in `codex-rs/app-server-client/src/lib.rs:189-190` and `codex-rs/app-server/src/in_process.rs:137-138`, and the TypeScript SDK injects `env.CODEX_API_KEY` in `sdk/typescript/src/exec.ts:160-162`. There is no dual-read or deprecation strategy for existing clients/scripts using `CODEX_API_KEY`. | Either keep `CODEX_API_KEY` as a permanent compatibility alias or define a dual-read migration (`ORBIT_CODE_API_KEY` first, `CODEX_API_KEY` fallback) across core, app-server, CLI, and SDKs. Document which env var is written in examples and how long the old one remains supported. |
| 5 | 2.3 | The public package/runtime rename path is still incomplete at the implementation level. The script updates Python package directories and `pyproject.toml`, but its Python/TS source rewrite rules do not update several critical constants: the Python runtime setup still hard-codes `PACKAGE_NAME = "codex-cli-bin"` and `REPO_SLUG = "openai/codex"` (`sdk/python/_runtime_setup.py:18-20`), and the TypeScript SDK still resolves `@openai/codex` / platform packages separately from `package.json` (`sdk/typescript/src/exec.ts:42-52`). That means the plan’s claimed package migration is only partial. | Add an explicit package-release matrix covering source imports, runtime package names, GitHub release/download coordinates, npm optional dependency resolution, and consumer migration docs. If the new package names are the target, update these constants deliberately and decide whether old package names remain published as shims. |

## Recommended Improvements (Should Consider)

| # | Section | Problem | Recommendation |
| - | ------- | ------- | -------------- |
| 1 | 2.7 | The plan now has better compatibility separation, but the implementation is still one large scripted pass plus cleanup. | Split the work into staged PRs by surface: internal Rust/Bazel naming, home-dir migration, SDK/package rename, docs/release tooling. That will keep regressions reviewable and make rollback practical. |
| 2 | 2.7 | The migration section introduces new public API names like `find_orbit_code_home()` without describing whether `find_codex_home()` remains available as a compatibility alias. | Prefer adding new helpers while keeping the old ones as deprecated wrappers during the transition, especially in shared utility crates used across the workspace. |
| 3 | 2.6 | The home-dir migration snippet suggests a recursive copy but does not define behavior for partial copies, symlinks, permissions, or what happens if both directories exist. | Specify migration invariants: copy vs move vs symlink, idempotency, conflict resolution, and recovery behavior after interrupted migration. |
| 4 | 2.8 | The verification section still instructs running the full `cargo nextest run` directly (`docs/orbit/plans/rename-codex-to-orbit-code.md:633-635`) even though root guidance requires asking before a complete suite run after shared/core/protocol changes (`AGENTS.md:44-49`). | Update the plan to gate the full-suite run behind explicit approval and keep targeted crate/package tests as the default automated validation path. |
| 5 | 2.4 | The plan now uses the right canonical `just` commands, but it should also spell out how generated docs/schema text mentioning `.codex` are intentionally handled during transition. | Add a generated-artifact checklist that distinguishes intentional compatibility mentions from strings that should be rebranded immediately. |

## Generated Artifacts and Docs To Update

- `codex-rs/app-server-protocol/schema/json/**` and `codex-rs/app-server-protocol/schema/typescript/**` via `just write-app-server-schema`
- `codex-rs/core/config.schema.json` via `just write-config-schema`
- hooks schema fixtures via `just write-hooks-schema`
- Python generated SDK artifacts under `sdk/python/src/codex_app_server/generated/**` if package/import shapes change
- npm and Python release tooling/docs: `codex-cli/scripts/build_npm_package.py`, `scripts/stage_npm_packages.py`, `sdk/python/_runtime_setup.py`, `sdk/typescript/src/exec.ts`, `codex-cli/bin/codex.js`
- docs mentioning persisted state/config under `~/.codex` or project `.codex/`, including `docs/config.md`, `docs/install.md`, `sdk/typescript/README.md`, and app-server docs/examples
- `MODULE.bazel.lock` if any Cargo manifests or lockfiles change
- TUI and `tui_app_server` snapshots if any user-visible branding changes land

## Edge Cases Not Addressed

- What happens if a repo already contains `.codex/config.toml` and `.orbit-code/config.toml` simultaneously? Which one wins, and how is that communicated?
- What happens if `~/.orbit-code` already exists but contains only partial migrated state while the old `~/.codex` still has newer auth/session data?
- What happens to keyring entries when both the service name and the derived account key may change after moving from `CODEX_HOME` / `.codex` to Orbit-branded paths?
- What happens to existing scripts, embedders, and SDK consumers that still set `CODEX_API_KEY`, import `codex_app_server`, or resolve `@openai/codex` packages?
- What happens when the Python runtime installer still downloads from `openai/codex` release coordinates but the new package/runtime names are published under Orbit branding?

## Verification Plan

- `cd codex-rs && cargo test -p codex-utils-home-dir`
- `cd codex-rs && cargo test -p codex-keyring-store`
- `cd codex-rs && cargo test -p codex-app-server-protocol`
- `cd codex-rs && cargo test -p codex-core`
- `cd codex-rs && cargo test -p codex-tui`
- `cd codex-rs && cargo test -p codex-tui-app-server`
- `cd codex-rs && just write-app-server-schema`
- `cd codex-rs && just write-config-schema`
- `cd /Users/no9labs/Developer/Recursive/codex && pnpm run write-hooks-schema`
- `cd sdk/python && pytest`
- `cd sdk/python-runtime && pytest`
- `cd sdk/typescript && pnpm build && pnpm test && pnpm lint`
- `cd shell-tool-mcp && pnpm build && pnpm test`
- If Cargo manifests or lockfiles change: `cd /Users/no9labs/Developer/Recursive/codex && just bazel-lock-update && just bazel-lock-check`
- Only after targeted suites pass, and with user approval per repo guidance: `cd codex-rs && cargo nextest run --no-fail-fast`

## Code Suggestions

The revised plan is close enough that I would not throw away the compatibility-surface structure. I would change the execution strategy instead:

1. Implement the home-dir transition first in `codex-utils-home-dir`, with dual-read support and tests before any textual rename of `.codex` references elsewhere.
2. Add auth/keyring compatibility next in `codex-core::auth`, keeping `CODEX_API_KEY` and old keyring service lookup as fallbacks.
3. Replace the shell-function-driven rename script with a deterministic tool or a much simpler script that does not rely on non-exported shell functions inside `bash -c`.
4. Only after migration shims exist, do the internal crate/import/branding rename pass.
5. Treat package renames as a release-track task with explicit consumer migration notes and, ideally, temporary compatibility packages.

## Verdict Details

### Correctness: CONCERNS

The plan now preserves several important wire contracts correctly, but the implementation details are not yet trustworthy. The script does not execute as written, and the migration only covers a subset of the old `.codex` behaviors currently encoded in the repo.

### Architecture: CONCERNS

The compatibility-surface decomposition is the right direction. The remaining architectural issue is that the implementation still tries to collapse several distinct migrations into one scripted pass instead of sequencing them by dependency and blast radius.

### Compatibility: CONCERNS

Compatibility is improved but still incomplete. `codex/sandbox-state` and sandbox env vars are preserved, which is correct, but project-local `.codex` behavior, `CODEX_API_KEY`, keyring service migration, and public package/runtime transitions still lack full compatibility semantics.

### Tooling: CONCERNS

The updated plan now uses the canonical `just` commands and includes Bazel lock maintenance, which is an improvement. The remaining tooling blocker is the broken script structure plus incomplete source coverage for package/runtime rename constants.

### Performance: PASS

There are no major steady-state performance concerns inherent in the new plan. The primary risk remains implementation churn and coordination cost, not runtime overhead.

### Production Readiness: CONCERNS

The revised plan is not yet safe to execute on a production codebase because the migration logic, auth/keyring fallback, and package/runtime cutover are not specified tightly enough to avoid data-loss or compatibility regressions.

### Test Coverage: CONCERNS

The targeted test list is better than before, especially with home-dir and keyring crates added conceptually, but the plan still needs to respect the repo’s approval requirement before complete test-suite execution and should add explicit tests for migration edge cases.
