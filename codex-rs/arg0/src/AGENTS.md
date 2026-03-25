# codex-rs/arg0/src/

This file applies to `codex-rs/arg0/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-arg0` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-arg0`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source code for the `codex-arg0` crate.

### What this folder does

Contains the single-file implementation of argv[0]-based dispatch and process bootstrapping.

### Key files

- `lib.rs` -- Complete crate implementation:
  - **Constants**: `LINUX_SANDBOX_ARG0`, `APPLY_PATCH_ARG0`, `EXECVE_WRAPPER_ARG0`, `LOCK_FILENAME`, `TOKIO_WORKER_STACK_SIZE_BYTES` (16 MB)
  - **Types**:
    - `Arg0DispatchPaths` -- Holds optional paths to `codex-linux-sandbox` and `codex-execve-wrapper` executables
    - `Arg0PathEntryGuard` -- RAII guard that keeps the temp directory and file lock alive for the process lifetime
  - **Functions**:
    - `arg0_dispatch()` -- Inspects argv[0]; if the binary name matches a known alias, dispatches directly to that tool (never returns). Otherwise, loads `.env`, creates PATH entry, and returns a guard.
    - `arg0_dispatch_or_else(main_fn)` -- Wraps `arg0_dispatch` and runs the given async closure on a Tokio runtime
    - `prepend_path_entry_for_codex_aliases()` -- Creates symlinks (Unix) or `.bat` wrappers (Windows) in a temp directory under `~/.orbit/tmp/arg0/`
    - `load_dotenv()` -- Loads `~/.orbit/.env`, filtering out `CODEX_`-prefixed variables for security
    - `janitor_cleanup(temp_root)` -- Removes stale temp directories whose file locks are not held
    - `build_runtime()` -- Creates Tokio multi-thread runtime with 16 MB stack

### Imports from / exports to

**Imports:**
- `codex_apply_patch::CODEX_CORE_APPLY_PATCH_ARG1`, `codex_apply_patch::apply_patch`, `codex_apply_patch::main`
- `codex_linux_sandbox::run_main`
- `codex_shell_escalation::run_shell_escalation_execve_wrapper`
- `codex_utils_home_dir::find_codex_home`

**Exports:**
- `Arg0DispatchPaths`, `Arg0PathEntryGuard`, `arg0_dispatch`, `arg0_dispatch_or_else`, `prepend_path_entry_for_codex_aliases`
