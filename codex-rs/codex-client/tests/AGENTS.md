# codex-rs/codex-client/tests/

This file applies to `codex-rs/codex-client/tests/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-client` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-client`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Tests for the `codex-client` crate.

### What this folder does

Contains tests for custom CA certificate handling, including environment variable-based cert loading and certificate chain validation.

### Key files

| File | Role |
|------|------|
| `ca_env.rs` | Tests for `NODE_EXTRA_CA_CERTS` environment variable handling and custom CA loading |
| `fixtures/` | Test certificate files |
