# codex-rs/arg0/

This file applies to `codex-rs/arg0/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-arg0` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-arg0`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Crate: `codex-arg0` -- Argv[0] dispatch and process bootstrapping for the Codex CLI.

### What this crate does

Implements the "arg0 trick" -- a technique where a single binary behaves differently depending on the name it was invoked as (argv[0]). This allows Codex to ship as one executable while exposing multiple internal tools (`apply_patch`, `codex-linux-sandbox`, `codex-execve-wrapper`) via symlinks.

Key responsibilities:
1. **Arg0 dispatch**: Checks the executable name and dispatches to the appropriate sub-tool (Linux sandbox, apply-patch, execve wrapper)
2. **PATH setup**: Creates a temporary directory with symlinks (Unix) or batch scripts (Windows) pointing back to the main executable, prepends it to PATH so child processes can find helper tools
3. **Dotenv loading**: Loads environment variables from `~/.orbit/.env` before threads are spawned (filtering out `CODEX_`-prefixed vars for security)
4. **Tokio runtime**: Builds a multi-thread Tokio runtime with 16 MB worker stack size
5. **Cleanup**: Implements a janitor that removes stale temporary directories from previous sessions using file locks

### Main functions

- `arg0_dispatch()` -- Checks argv[0] and dispatches to sub-tools if matched; otherwise sets up PATH and returns a guard
- `arg0_dispatch_or_else(main_fn)` -- Primary entry point for binary crates; dispatches or runs the provided async main function
- `prepend_path_entry_for_codex_aliases()` -- Creates temp dir with symlinks and prepends to PATH

### What it plugs into

- Called from `main()` in binary crates (`cli/`, `exec/`, `tui/`) as the first thing before any async work
- The temporary PATH entry allows `codex-core` to spawn sandbox and apply-patch sub-processes

### Imports from / exports to

**Dependencies:**
- `codex-apply-patch` -- Apply-patch tool implementation
- `codex-linux-sandbox` -- Linux sandbox implementation
- `codex-shell-escalation` -- Execve wrapper for shell escalation
- `codex-utils-home-dir` -- Finds `~/.orbit` home directory
- `dotenvy` -- `.env` file loading
- `tempfile` -- Temporary directory creation
- `tokio` -- Async runtime

**Exports:**
- `Arg0DispatchPaths` -- Struct containing paths to helper executables
- `Arg0PathEntryGuard` -- RAII guard keeping the temp directory alive
- `arg0_dispatch()`, `arg0_dispatch_or_else()` -- Entry points

### Key files

- `Cargo.toml` -- Crate manifest with dependencies
- `src/lib.rs` -- Complete implementation (single file)
