# codex-rs/process-hardening/

Pre-main security hardening for the Codex process. Prevents debugging, memory inspection, core dumps, and library injection attacks.

## Build & Test
```bash
cargo build -p orbit-code-process-hardening
cargo test -p orbit-code-process-hardening
```

## Architecture

A single public function `pre_main_hardening()` dispatches to platform-specific hardening routines. On Linux: `prctl(PR_SET_DUMPABLE, 0)` + core dump disable + `LD_*` env removal. On macOS: `ptrace(PT_DENY_ATTACH)` + core dump disable + `DYLD_*` env removal. On BSD: core dump disable + `LD_*` env removal. Called from `#[ctor::ctor]` in the main binary to run before `main()`.

## Key Considerations
- All hardening functions **exit the process immediately** on failure with distinct exit codes (5, 6, 7) -- they never continue in a vulnerable state
- Environment variable filtering handles non-UTF-8 keys correctly (filters by byte prefix via `OsStr`)
- This crate only depends on `libc` -- keep it minimal since it runs before `main()`
- The crate is single-file (`src/lib.rs`) -- no modules
- Tests verify env var filtering edge cases including non-UTF-8 keys
