# codex-rs/core/src/features/

This file applies to `codex-rs/core/src/features/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-core` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-core`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Legacy feature flag aliases and migration support.

### What this folder does

This subdirectory contains the legacy feature toggle mapping system. It maps old config keys (e.g., `connectors`, `include_apply_patch_tool`, `experimental_use_unified_exec_tool`) to the current canonical `Feature` enum variants.

The main `features.rs` file (in the parent directory) defines the `Feature` enum, `Features` container, and feature metadata (stage, default value, description). This `features/` subdirectory specifically handles backward compatibility.

### Key files

| File | Purpose |
|------|---------|
| `legacy.rs` | `ALIASES` table mapping old config keys to `Feature` variants; `LegacyFeatureToggles` struct for applying deprecated toggle fields; `feature_for_key()` lookup |

### Key concepts

- **Feature aliases**: Old keys like `"connectors"` map to `Feature::Apps`, `"web_search"` maps to `Feature::WebSearchRequest`, etc.
- **Legacy toggles**: The `LegacyFeatureToggles` struct holds deprecated boolean fields (`include_apply_patch_tool`, `experimental_use_freeform_apply_patch`, `experimental_use_unified_exec_tool`) and applies them to a `Features` container with deprecation logging.
- **Migration logging**: When a legacy key is used, an info log is emitted suggesting the canonical `[features].{key}` form.

### Imports from

- `crate::features` (parent) -- `Feature`, `Features` types

### Exports to

- `crate::features` -- re-exports `LegacyFeatureToggles`, `legacy_feature_keys()`, `feature_for_key()`
- `crate::config` -- uses legacy toggles during config deserialization
