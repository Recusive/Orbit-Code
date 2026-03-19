# codex-rs/execpolicy/examples/

This file applies to `codex-rs/execpolicy/examples/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-execpolicy` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-execpolicy`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Example policy files demonstrating the `.codexpolicy` Starlark DSL syntax.

### What this folder does

Contains example exec policy files that illustrate the syntax and capabilities of the policy language. These are reference examples, not recommended for production use.

### Key files and their roles

- `example.codexpolicy` -- Demonstrates `prefix_rule()` with various configurations: forbidden rules with justification (`git reset --hard`), allow rules (`ls`, `cat`, `pwd`, `which`, `printenv`), prompt rules (`cp`), and match/not_match example annotations for self-testing rules.
