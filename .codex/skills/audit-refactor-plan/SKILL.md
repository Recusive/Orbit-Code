---
name: audit-refactor-plan
description: Audit proposed refactor plans for production readiness in the Orbit Code monorepo. Use when asked to evaluate a plan document for correctness, architecture, crate boundaries, protocol and SDK compatibility, Bazel/Cargo/pnpm tooling fit, migration risk, edge cases, and test coverage before implementation.
---

# Audit Refactor Plan

Perform a comprehensive plan review before code is written. Treat this like a production design audit, not a summary.

## Input and validation

Require a plan document path.

If the path is missing, output exactly:

```text
Usage: /audit-plan <path-to-plan-doc>
Example: /audit-plan docs/exit-confirmation-prompt-design.md
```

Stop execution.

If the path does not exist, output exactly:

```text
Plan document not found at `<path-to-plan-doc>`
```

Stop execution.

## Context

Collect context first:

- Current branch: `git branch --show-current`
- Repository assumptions: Orbit Code monorepo with Rust-first implementation in `codex-rs/`, plus Python and TypeScript SDKs, Bazel support, pnpm workspace packages, docs, scripts, and generated schemas.
- Read the root `AGENTS.md` and `CLAUDE.md`.
- Read the nearest subsystem `AGENTS.md` and `CLAUDE.md` files for every area the plan touches.

Keep the review anchored in repo conventions, not generic preferences.

## Step 0: Understand the plan

Read the plan document fully. Extract:

1. What subsystem or workflow is being changed
2. The goal of the refactor
3. The current implementation files mentioned or implied
4. The proposed new files, crates, modules, APIs, or migrations
5. Cross-cutting concerns: config, protocol, persistence, SDKs, docs, generated artifacts, and rollout strategy
6. Similar reference patterns already present in the repo

## Step 1: Read current implementations

Read every file the plan proposes to change or replace. Also read:

- Direct imports and dependencies of those files
- At least one similar existing pattern in the same subsystem
- Relevant build, schema, and documentation files when the plan changes contracts

Use the following repo-specific prompts while gathering context:

- If the plan touches `codex-rs/tui`, also read `codex-rs/tui/styles.md` and check whether `codex-rs/tui_app_server` needs parallel changes.
- If the plan touches app-server or protocol work, read `codex-rs/app-server/README.md`, `codex-rs/app-server-protocol/src/protocol/common.rs`, and `codex-rs/app-server-protocol/src/protocol/v2.rs`.
- If the plan touches config, read the relevant config types and `codex-rs/core/config.schema.json`.
- If the plan touches SDKs, read the corresponding SDK docs, tests, and generated artifacts.
- If the plan introduces build-time file reads such as `include_str!`, `include_bytes!`, or migrations, inspect the relevant `BUILD.bazel`.
- If the plan touches workspace packaging, inspect `package.json`, `pnpm-workspace.yaml`, and any package-level manifests.

Stop reading only when you can explain:

- Current behavior and ownership boundaries
- Inputs, outputs, and side effects
- Async, persistence, and error-handling boundaries
- Existing conventions the change is expected to follow
- Which artifacts or docs would need regeneration

## Step 2: Audit against criteria

Evaluate every applicable section. Skip only sections that do not apply, and explicitly state what was skipped and why.

### 2.1 Correctness and behavioral compatibility

- Verify the proposed abstraction still covers the current behavior.
- Identify any behavior that would diverge across CLI, TUI, app-server, exec, MCP, or SDK surfaces.
- Identify state migration risks for sessions, threads, config, auth, persistence, or cached data.
- Identify rollout or fallback gaps if the old and new systems must coexist.
- Call out any place where the plan assumes behavior that current code does not actually provide.

### 2.2 Architecture and code organization

- Evaluate whether the proposed boundary belongs in a crate, module, service, protocol type, hook, or utility.
- Check whether the plan respects existing crate ownership and avoids pushing more logic into already hot central modules.
- Recommend extracting new modules instead of growing large files when applicable.
- Check whether shared behavior belongs in common/core/protocol versus leaf crates.
- If the plan touches `tui`, verify whether `tui_app_server` needs the same behavior.

### 2.3 API, protocol, and SDK compatibility

- Verify whether any Rust, JSON-RPC, CLI, Python, or TypeScript contract changes are proposed.
- For app-server work, enforce v2-only API growth unless the plan is explicitly about compatibility maintenance.
- Check naming, payload shape, pagination, timestamps, and experimental-field handling against repo conventions.
- Identify downstream SDK, generated-schema, or docs updates that the plan omits.
- Call out compatibility risks for plugins, MCP integrations, hooks, or automation consumers when relevant.

### 2.4 Tooling and build-system fit

- Evaluate Cargo, Bazel, and pnpm implications.
- Check whether dependency changes imply `MODULE.bazel.lock` refresh and verification.
- Check whether config or app-server changes imply schema regeneration.
- Check whether build-time file access requires `BUILD.bazel` data updates.
- Evaluate whether the plan introduces generation steps, fixtures, or scripts without documenting how they stay in sync.

### 2.5 Performance and concurrency

- Compare current versus proposed startup, render, I/O, and memory behavior.
- Identify unnecessary cloning, channel fan-out, blocking work on async paths, or increased TUI redraw cost.
- Identify race conditions, ordering hazards, or lag/backpressure risks for event streams and background tasks.
- Evaluate persistence and database access patterns when the plan changes state flow.
- Call out where a simpler architecture would reduce coordination cost.

### 2.6 Production readiness and cross-platform behavior

- Evaluate error handling, fallback behavior, and observability.
- Check macOS seatbelt, Linux sandbox, Windows support, and environment-sensitive behavior when applicable.
- Identify at least 3-5 concrete edge cases the plan does not address.
- Verify that the plan preserves strict typing and avoids hand-wavy compatibility claims.
- Call out any unsafe migration strategy for user data, auth state, or on-disk artifacts.

### 2.7 Extensibility and maintainability

- Evaluate whether the proposed design can support likely follow-on work without immediate rework.
- Check whether the plan introduces ambiguous APIs such as bool or bare `Option` parameters where a clearer shape is warranted.
- Decide whether interfaces, schemas, or traits should be defined now versus later.
- Check whether docs, examples, and subsystem guides need updates for future contributors.

### 2.8 Test and verification strategy

- Identify current tests that already protect the area.
- Recommend the minimum crate-specific test commands needed after implementation.
- If the plan changes TUI output, require snapshot coverage and snapshot review.
- If the plan changes protocol or config shapes, require schema regeneration and targeted protocol tests.
- If the plan touches shared crates such as common, core, or protocol, note that the eventual implementation should plan for a broader test pass after targeted tests.
- Recommend SDK or packaging verification when public interfaces change.

## Step 3: Write the audit report

Write the report to `reviews/<plan-stem>.audit.md` using this structure:

```markdown
# Plan Audit: [Plan Title]

**Date**: [current date]
**Plan Document**: [path-to-plan-doc]
**Branch**: [branch name]

## Plan Summary

[2-3 sentences: what the refactor changes, what it replaces, and the stated goal]

## Files Reviewed

| File | Role | Risk |
| ---- | ---- | ---- |
| `path/to/file` | Current implementation | High |
| `path/to/file` | Reference pattern | Low |

_Risk: High (core contract or shared subsystem), Medium (feature code or adapter), Low (leaf utility, tests, docs)_

## Verdict: [APPROVE / APPROVE WITH CHANGES / NEEDS REWORK]

[1-2 sentence justification]

## Critical Issues (Must Fix Before Implementation)

| # | Section | Problem | Recommendation |
| - | ------- | ------- | -------------- |
| 1 | 2.3 | [What's wrong] | [How to fix] |

## Recommended Improvements (Should Consider)

| # | Section | Problem | Recommendation |
| - | ------- | ------- | -------------- |
| 1 | 2.5 | [What could be better] | [Suggestion] |

## Generated Artifacts and Docs To Update

- [Required schema, lockfile, snapshot, SDK, or docs updates]

## Edge Cases Not Addressed

- What happens if X?
- What happens when Y?

## Verification Plan

- `cargo test -p ...`
- `just write-config-schema`
- `just write-app-server-schema`
- `cargo insta pending-snapshots -p codex-tui`

## Code Suggestions

[Concrete code-level suggestions for the highest-risk issues]

## Verdict Details

### Correctness: [PASS / CONCERNS]

[Details]

### Architecture: [PASS / CONCERNS]

[Details]

### Compatibility: [PASS / CONCERNS]

[Details]

### Tooling: [PASS / CONCERNS]

[Details]

### Performance: [PASS / CONCERNS]

[Details]

### Production Readiness: [PASS / CONCERNS]

[Details]

### Test Coverage: [PASS / CONCERNS]

[Details]
```

## Step 4: Final output contract

After writing the report:

1. Print `Audit written to reviews/<plan-stem>.audit.md`
2. Print the `Verdict`, `Critical Issues`, `Generated Artifacts and Docs To Update`, and `Edge Cases Not Addressed` sections

## Policies

- Be thorough; this repo has shared contracts across multiple surfaces.
- Do not rubber-stamp; justify approval with evidence from files already read.
- Be concrete; reference files, lines, crates, commands, and generated artifacts.
- Respect existing repo conventions unless there is a strong reason to deviate.
- If a plan omits migration, rollback, schema, snapshot, or docs implications, treat that as a real gap rather than a minor note.
