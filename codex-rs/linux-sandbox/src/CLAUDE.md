# codex-rs/linux-sandbox/src/

Source directory for the `orbit-code-linux-sandbox` crate. All modules are conditionally compiled for `target_os = "linux"` only.

## What this folder does

Contains the implementation of the Linux sandbox helper binary and library. The code applies filesystem isolation via bubblewrap and network restrictions via seccomp/Landlock, then exec's the target command.

## Architecture

The sandbox operates in a two-stage pipeline:

1. **Outer stage** (`linux_run_main.rs`): Wraps the command with bubblewrap to establish the filesystem namespace, then re-enters itself as the inner stage.
2. **Inner stage** (`linux_run_main.rs` with `--apply-seccomp-then-exec`): Applies seccomp filters and `no_new_privs`, optionally activates proxy routing bridges, then `execvp`'s into the user command.

## Key files and their roles

| File | Purpose |
|------|---------|
| `lib.rs` | Library entry point; declares modules, exports `run_main()` |
| `main.rs` | Binary entry point; calls `orbit_code_linux_sandbox::run_main()` |
| `bwrap.rs` | Generates bubblewrap CLI argument vectors from `FileSystemSandboxPolicy`. Handles mount ordering: read-only root, `/dev`, writable roots, read-only subpaths, unreadable carveouts. Includes `BwrapOptions` and `BwrapNetworkMode` types |
| `landlock.rs` | Applies in-process sandbox primitives: `PR_SET_NO_NEW_PRIVS` via `prctl`, seccomp BPF filters for network syscall restriction (Restricted and ProxyRouted modes), and legacy Landlock filesystem rules |
| `launcher.rs` | Decides between system bwrap (`/usr/bin/bwrap`) and vendored bwrap; handles `execv` and fd inheritance (`clear_cloexec`) for preserved files |
| `linux_run_main.rs` | CLI surface (`LandlockCommand` via clap); policy resolution logic; two-stage execution orchestration; `/proc` mount preflight probing |
| `linux_run_main_tests.rs` | Unit tests for policy resolution and inner-command building |
| `proxy_routing.rs` | Managed proxy routing: discovers proxy environment variables, spawns UDS-to-TCP host bridges, activates local TCP-to-UDS bridges inside the network namespace, rewrites proxy env vars |
| `vendored_bwrap.rs` | FFI wrapper for the statically linked `bwrap_main` C function; falls back to a panic when the vendored build is unavailable |

## Imports / exports

- **Imports from workspace**: `orbit-code-core` (error types), `orbit-code-protocol` (sandbox policy types), `orbit-code-utils-absolute-path`
- **External deps**: `clap`, `landlock`, `seccompiler`, `libc`, `serde`, `serde_json`, `url`
- **Exports**: `run_main()` function (the binary entry point)
