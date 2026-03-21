# codex-rs/execpolicy/

Prefix-based Starlark policy engine that evaluates whether shell commands should be allowed, prompted, or forbidden. This is the current (non-legacy) exec policy system.

## Build & Test
```bash
cargo build -p orbit-code-execpolicy
cargo test -p orbit-code-execpolicy
```

## Architecture

Policies are written in a Starlark DSL (`.codexpolicy` files) using built-in functions like `prefix_rule()`, `network_rule()`, and `host_executable()`. The `PolicyParser` evaluates these files via the Starlark interpreter and builds a `Policy` object. The `Policy` stores rules indexed by program name in a `MultiMap` and evaluates commands by tokenizing them with `shlex` and matching against prefix patterns. Each evaluation returns a `Decision` (Allow, Prompt, or Forbidden) along with the matched rules.

The crate also provides a CLI binary (`orbit-code-execpolicy check`) for manual policy evaluation, and helpers for appending new rules to existing policy files on disk.

## Key Considerations
- `Decision` has an ordering: `Allow < Prompt < Forbidden` -- the most restrictive decision wins when multiple rules match
- `PrefixPattern` supports alternates via `PatternToken::Alts` for matching multiple argument options in a single rule
- Rule match/not-match examples are validated at parse time -- if a rule's examples fail, the policy file is rejected
- The `merge_overlay()` method layers policies (e.g., project-level over global) -- overlay rules take precedence
- The binary also exposes the library as `orbit_code_execpolicy`
- `include_str!` is not used here, but `default.policy` lives in the legacy crate -- do not confuse the two
