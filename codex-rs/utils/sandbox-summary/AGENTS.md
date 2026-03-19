# codex-rs/utils/sandbox-summary/

This file applies to `codex-rs/utils/sandbox-summary/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-utils-sandbox-summary` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-utils-sandbox-summary`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Crate `codex-utils-sandbox-summary` -- human-readable summaries of sandbox and configuration policies.

### What this folder does

Generates human-readable summary strings for sandbox policies and configuration settings, used in the TUI status bar and other display contexts.

### Key types and functions

- `summarize_sandbox_policy(policy: &SandboxPolicy) -> String` -- produces a compact summary like `"workspace-write [workdir, /tmp, $TMPDIR] (network access enabled)"` for any sandbox policy variant
- `create_config_summary_entries(config, model) -> Vec<(&str, String)>` -- builds key/value pairs summarizing the effective config (workdir, model, provider, approval, sandbox, reasoning effort/summaries)

### Imports from

- `codex-core` -- `Config`, `WireApi`
- `codex-protocol` -- `SandboxPolicy`, `NetworkAccess`

### Exports to

Used by `codex-tui` for displaying configuration status and by other UI surfaces.

### Key files

- `Cargo.toml` -- crate metadata; depends on `codex-core`, `codex-protocol`
- `src/lib.rs` -- module declarations and re-exports
- `src/sandbox_summary.rs` -- `summarize_sandbox_policy` with variant-specific formatting
- `src/config_summary.rs` -- `create_config_summary_entries` for building config display data
