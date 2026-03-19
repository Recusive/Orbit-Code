# codex-rs/process-hardening/

Crate: `codex-process-hardening` -- Pre-main security hardening for the Codex process.

## What this crate does

Performs platform-specific security hardening steps that run before `main()` (via `#[ctor::ctor]`). These measures protect the Codex process from debugging, memory inspection, and environment variable manipulation attacks.

## Main function

- `pre_main_hardening()` -- Dispatches to platform-specific hardening:
  - **Linux/Android**:
    - `prctl(PR_SET_DUMPABLE, 0)` -- Marks process as non-dumpable, preventing ptrace attach
    - `setrlimit(RLIMIT_CORE, 0)` -- Disables core dumps
    - Removes all `LD_*` environment variables (e.g., `LD_PRELOAD`) to prevent library injection
  - **macOS**:
    - `ptrace(PT_DENY_ATTACH)` -- Prevents debugger attachment
    - `setrlimit(RLIMIT_CORE, 0)` -- Disables core dumps
    - Removes all `DYLD_*` environment variables to prevent dylib injection
  - **FreeBSD/OpenBSD**:
    - `setrlimit(RLIMIT_CORE, 0)` -- Disables core dumps
    - Removes all `LD_*` environment variables
  - **Windows**: Placeholder (TODO)

## Key behaviors

- All hardening functions exit the process immediately on failure (with distinct exit codes: 5, 6, 7) rather than continuing in a vulnerable state
- Environment variable filtering handles non-UTF-8 keys correctly (filters by byte prefix)
- The `env_keys_with_prefix` helper is generic over any iterator of `(OsString, OsString)` pairs

## What it plugs into

- Called from `#[ctor::ctor]` in the main binary crate (`cli/`) to run before `main()`
- Ensures that the process handling API keys and executing code is resistant to common local attacks

## Imports from / exports to

**Dependencies:**
- `libc` -- System calls (`prctl`, `ptrace`, `setrlimit`)

**Exports:**
- `pre_main_hardening()` -- The sole public function

## Key files

- `Cargo.toml` -- Crate manifest
- `src/lib.rs` -- Single-file implementation with platform-conditional compilation
