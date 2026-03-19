# codex-rs/windows-sandbox-rs/src/

This file applies to `codex-rs/windows-sandbox-rs/src/` and its descendants. Follow the repo root `AGENTS.md` first, then use the local rules below when you edit this subtree.

## Agent Guidance
- Follow the repo-root Rust rules in `/Users/no9labs/Developer/Recursive/codex/AGENTS.md`: keep modules focused, prefer exhaustive matches, and avoid touching sandbox-env handling unless the task explicitly requires it.
- This subtree belongs to the `codex-windows-sandbox` crate. Keep public re-exports, module wiring, and tests in sync with any behavior changes here.

## Validate
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && just fmt`
- `cd /Users/no9labs/Developer/Recursive/codex/codex-rs && cargo test -p codex-windows-sandbox`

## Directory Map
The summary below is based on the existing directory documentation and cross-checked against the files currently present here.

Source code for the `codex-windows-sandbox` crate.

### What this folder does

Contains the full Windows sandbox implementation. All Windows-specific modules are conditionally compiled with `#[cfg(target_os = "windows")]`. On other platforms, stub implementations return errors.

### Key files

- `lib.rs` -- top-level module declarations using a `windows_modules!` macro, conditional ConPTY/elevated/setup imports, the main `run_windows_sandbox_capture()` implementation (token creation, ACL manipulation, process spawning, output capture), and non-Windows stubs.
- `acl.rs` -- Windows ACL (Access Control List) manipulation: add/remove allow/deny ACEs.
- `allow.rs` -- computes allowed/denied filesystem paths based on sandbox policy.
- `audit.rs` -- world-writable scan and deny ACE application.
- `cap.rs` -- capability SID management for sandbox tokens.
- `desktop.rs` -- private desktop creation for sandboxed processes.
- `dpapi.rs` -- DPAPI encryption/decryption for secrets.
- `env.rs` -- environment variable sanitization (network blocking, null device, pager).
- `firewall.rs` -- Windows Firewall rule management via COM/WMI.
- `identity.rs` -- sandbox user credential management.
- `policy.rs` -- `SandboxPolicy` enum and parser (ReadOnly, WorkspaceWrite, DangerFullAccess, ExternalSandbox).
- `process.rs` -- `create_process_as_user()`, pipe-based process spawning, handle reading.
- `token.rs` -- restricted token creation with capability SIDs.
- `workspace_acl.rs` -- workspace-specific ACL protection.
- `setup_orchestrator.rs` -- elevated setup orchestration.
- `setup_error.rs` -- setup error types and reporting.
- `elevated_impl.rs` -- elevated sandbox capture implementation.

### Subdirectories

- `bin/` -- binary entrypoints.
- `conpty/` -- ConPTY helpers.
- `elevated/` -- elevated runner IPC and implementation.
