# codex-rs/execpolicy-legacy/tests/suite/

Integration test modules for the legacy exec policy engine.

## What this folder does

Contains individual test modules that verify the behavior of the legacy argument-aware exec policy engine. Each module focuses on a specific program or feature of the policy system.

## Key files and their roles

- `mod.rs` -- Module aggregator; imports all test modules.
- `good.rs` -- Tests for commands that should pass policy checks (safe commands).
- `bad.rs` -- Tests for commands that should fail or be flagged.
- `cp.rs` -- Tests specific to `cp` command policy rules.
- `head.rs` -- Tests specific to `head` command policy rules.
- `ls.rs` -- Tests specific to `ls` command policy rules.
- `pwd.rs` -- Tests specific to `pwd` command policy rules.
- `sed.rs` -- Tests for sed command policy rules.
- `parse_sed_command.rs` -- Tests for the sed command parser (`parse_sed_command()`).
- `literal.rs` -- Tests for literal/exact matching behavior.

## What it plugs into

- Aggregated by `tests/all.rs` via `mod suite;`
