# codex-rs/codex-client/tests/fixtures/

This file applies to `codex-rs/codex-client/tests/fixtures/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Treat this directory as golden data or generated/static support material. Keep filenames and relative paths stable unless the owning test, renderer, or generator changes first.
- Prefer updating the producer or the corresponding test scenario before editing files here by hand.
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-client` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-client`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Test certificates for custom CA handling tests.

### What this folder does

Contains PEM-encoded test certificates used by `ca_env.rs` tests to verify custom CA certificate loading and chain validation behavior.

### Key files

| File | Role |
|------|------|
| `test-ca.pem` | Self-signed test CA certificate |
| `test-ca-trusted.pem` | Trusted test CA certificate |
| `test-intermediate.pem` | Intermediate certificate for chain validation tests |
