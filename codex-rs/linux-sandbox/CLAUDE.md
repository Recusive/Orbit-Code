# codex-rs/linux-sandbox/

Linux-specific process sandbox for the Codex CLI. Ships as both a library (`orbit_code_linux_sandbox`) and a standalone binary (`orbit-code-linux-sandbox`).

## What this folder does

Provides filesystem and network isolation for commands executed by the Codex agent on Linux. It composes three complementary mechanisms:

1. **Bubblewrap (bwrap)** -- constructs a restricted filesystem namespace with read-only defaults, explicit writable roots, and protected subpaths (`.git`, `.orbit`).
2. **Seccomp** -- installs a BPF filter that blocks or restricts network-related syscalls (`connect`, `socket`, `bind`, etc.) depending on the policy.
3. **Landlock** -- legacy/backup filesystem restriction kept for fallback use.

The binary is invoked by `orbit-code-core` as a helper process when spawning sandboxed commands on Linux.

## Where it plugs in

- **Consumed by**: `orbit-code-core` (`exec::process_exec_tool_call`) invokes the `orbit-code-linux-sandbox` binary to wrap child commands.
- **Depends on**: `orbit-code-core` (error types), `orbit-code-protocol` (sandbox policy types like `SandboxPolicy`, `FileSystemSandboxPolicy`, `NetworkSandboxPolicy`), `orbit-code-utils-absolute-path`, `landlock`, `seccompiler`, `libc`.

## Key files

| File | Role |
|------|------|
| `Cargo.toml` | Crate definition; linux-only dependencies behind `cfg(target_os = "linux")` |
| `build.rs` | Compiles vendored bubblewrap C sources via `cc` for the embedded bwrap path |
| `config.h` | C header consumed by the vendored bubblewrap build |
| `src/main.rs` | Binary entry point; delegates to `run_main()` |
| `src/lib.rs` | Library root; conditionally compiles all modules on Linux, panics on other platforms |
| `src/bwrap.rs` | Builds bubblewrap CLI arguments from `FileSystemSandboxPolicy`; handles mount ordering for writable/unreadable/read-only roots |
| `src/landlock.rs` | Applies `no_new_privs`, seccomp network filters, and (legacy) Landlock filesystem rules on the current thread |
| `src/launcher.rs` | Chooses between system `/usr/bin/bwrap` and vendored bwrap; exec's into bubblewrap |
| `src/linux_run_main.rs` | CLI argument parsing (`LandlockCommand`) and two-stage sandbox setup (outer bwrap + inner seccomp) |
| `src/proxy_routing.rs` | Managed network proxy bridge: creates UDS-to-TCP bridges so sandboxed processes can reach a host-side proxy through an isolated network namespace |
| `src/vendored_bwrap.rs` | FFI wrapper around the statically compiled `bwrap_main` C symbol |
