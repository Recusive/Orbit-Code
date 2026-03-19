# codex-rs/utils/sleep-inhibitor/src/

This file applies to `codex-rs/utils/sleep-inhibitor/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-utils-sleep-inhibitor` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-utils-sleep-inhibitor`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source directory for the `codex-utils-sleep-inhibitor` crate.

### Key files

- `lib.rs` -- public API:
  - `SleepInhibitor` struct with fields: `enabled`, `turn_running`, `platform` (platform-specific impl)
  - `new(enabled: bool)` -- constructor
  - `set_turn_running(bool)` -- acquires/releases sleep prevention based on enabled + turn state
  - `is_turn_running()` -- getter
  - Conditional compilation selects the platform backend via `use ... as imp`
  - Tests for toggle behavior, disabled mode, idempotent calls, and multiple toggles
- `macos.rs` -- macOS implementation:
  - `MacSleepAssertion` -- creates `IOPMAssertionCreateWithName` with `PreventUserIdleSystemSleep`; releases on `Drop`
  - Wraps IOKit FFI from `iokit_bindings.rs` via `core_foundation::string::CFString`
- `iokit_bindings.rs` -- generated IOKit FFI bindings (included by macos.rs)
- `linux_inhibitor.rs` -- Linux implementation using subprocess inhibitors
- `windows_inhibitor.rs` -- Windows implementation using `PowerCreateRequest`/`PowerSetRequest`
- `dummy.rs` -- no-op implementation for unsupported platforms
